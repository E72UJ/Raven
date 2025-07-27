use enum_map::Enum;
use swf::{CharacterId, FillStyle, LineStyle, Rectangle, Shape, ShapeRecord, Twips};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FillRule {
    EvenOdd,
    NonZero,
}

impl From<FillRule> for lyon_tessellation::path::FillRule {
    fn from(rule: FillRule) -> lyon_tessellation::path::FillRule {
        match rule {
            FillRule::EvenOdd => lyon_tessellation::path::FillRule::EvenOdd,
            FillRule::NonZero => lyon_tessellation::path::FillRule::NonZero,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Enum, Hash)]
pub enum GradientType {
    Linear,
    Radial,
    Focal,
}

/// `DrawPath` represents a solid fill or a stroke.
/// Fills are always closed paths, while strokes may be open or closed.
/// Closed paths will have the first point equal to the last point.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawPath<'a> {
    Stroke {
        style: &'a LineStyle,
        is_closed: bool,
        commands: Vec<DrawCommand>,
    },
    Fill {
        style: &'a FillStyle,
        commands: Vec<DrawCommand>,
        winding_rule: FillRule,
    },
}

/// `DistilledShape` represents a ready-to-be-consumed collection of paths (both fills and strokes)
/// that has been converted down from another source (such as SWF's `swf::Shape` format).
#[derive(Clone, Debug, PartialEq)]
pub struct DistilledShape<'a> {
    pub paths: Vec<DrawPath<'a>>,
    pub shape_bounds: Rectangle<Twips>,
    pub edge_bounds: Rectangle<Twips>,
    pub id: CharacterId,
}

impl<'a> From<&'a swf::Shape> for DistilledShape<'a> {
    fn from(shape: &'a Shape) -> Self {
        Self {
            paths: ShapeConverter::from_shape(shape).into_commands(),
            shape_bounds: shape.shape_bounds.clone(),
            edge_bounds: shape.edge_bounds.clone(),
            id: shape.id,
        }
    }
}

/// `DrawCommands` trace the outline of a path.
/// Fills follow the even-odd fill rule, with opposite winding for holes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DrawCommand {
    MoveTo(swf::Point<Twips>),
    LineTo(swf::Point<Twips>),
    QuadraticCurveTo {
        control: swf::Point<Twips>,
        anchor: swf::Point<Twips>,
    },
    CubicCurveTo {
        control_a: swf::Point<Twips>,
        control_b: swf::Point<Twips>,
        anchor: swf::Point<Twips>,
    },
}

impl DrawCommand {
    pub fn end_point(&self) -> swf::Point<Twips> {
        match self {
            DrawCommand::MoveTo(point)
            | DrawCommand::LineTo(point)
            | DrawCommand::QuadraticCurveTo { anchor: point, .. }
            | DrawCommand::CubicCurveTo { anchor: point, .. } => *point,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Point {
    x: Twips,
    y: Twips,
    is_bezier_control: bool,
}

impl From<Point> for swf::Point<Twips> {
    fn from(point: Point) -> Self {
        Self::new(point.x, point.y)
    }
}

/// A continuous series of edges in a path.
/// Fill segments are directed, because the winding determines the fill-rule.
/// Stroke segments are undirected.
#[derive(Clone, Debug)]
struct PathSegment {
    pub points: Vec<Point>,
}

impl PathSegment {
    fn new(start: swf::Point<Twips>) -> Self {
        Self {
            points: vec![Point {
                x: start.x,
                y: start.y,
                is_bezier_control: false,
            }],
        }
    }

    fn reset(&mut self, start: swf::Point<Twips>) {
        self.points.clear();
        self.points.push(Point {
            x: start.x,
            y: start.y,
            is_bezier_control: false,
        });
    }

    /// Flips the direction of the path segment.
    /// Flash fill paths are dual-sided, with fill style 1 indicating the positive side
    /// and fill style 0 indicating the negative. We have to flip fill style 0 paths
    /// in order to link them to fill style 1 paths.
    fn flip(&mut self) {
        self.points.reverse();
    }

    /// Adds an edge to the end of the path segment.
    fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }

    fn is_empty(&self) -> bool {
        self.points.len() <= 1
    }

    fn start(&self) -> Option<swf::Point<Twips>> {
        Some((*self.points.first()?).into())
    }

    fn end(&self) -> Option<swf::Point<Twips>> {
        Some((*self.points.last()?).into())
    }

    fn is_closed(&self) -> bool {
        self.start() == self.end()
    }

    fn to_draw_commands(&self) -> impl '_ + Iterator<Item = DrawCommand> {
        assert!(!self.is_empty());
        let mut i = self.points.iter();
        let first = i.next().expect("Points should not be empty");
        std::iter::once(DrawCommand::MoveTo((*first).into())).chain(std::iter::from_fn(move || {
            match i.next() {
                Some(
                    point @ Point {
                        is_bezier_control: false,
                        ..
                    },
                ) => Some(DrawCommand::LineTo((*point).into())),
                Some(
                    point @ Point {
                        is_bezier_control: true,
                        ..
                    },
                ) => {
                    let end = i.next().expect("Bezier without endpoint");
                    Some(DrawCommand::QuadraticCurveTo {
                        control: (*point).into(),
                        anchor: (*end).into(),
                    })
                }
                None => None,
            }
        }))
    }
}

/// The internal path structure used by ShapeConverter.
///
/// Each path is uniquely identified by its fill/stroke style. But Flash gives
/// the path edges as an "edge soup" -- they can arrive in an arbitrary order.
/// We have to link the edges together for each path. This structure contains
/// a list of path segment, and each time a path segment is added, it will try
/// to merge it with an existing segment.
#[derive(Clone, Debug)]
struct PendingPath {
    /// The list of path segments for this fill/stroke.
    /// For fills, this should turn into a list of closed paths when the shape is complete.
    /// Strokes may or may not be closed.
    segments: Vec<PathSegment>,
}

impl PendingPath {
    fn new() -> Self {
        Self { segments: vec![] }
    }

    /// Adds a path segment to the path, attempting to link it to existing segments.
    fn add_segment(&mut self, mut new_segment: PathSegment) {
        if !new_segment.is_empty() {
            // Try to link this segment onto existing segments with a matching endpoint.
            // Both the start and the end points of the new segment can be linked.
            let mut start_open = true;
            let mut end_open = true;
            let mut i = 0;
            while (start_open || end_open) && i < self.segments.len() {
                let other = &mut self.segments[i];
                if start_open && other.end() == new_segment.start() {
                    other.points.extend_from_slice(&new_segment.points[1..]);
                    new_segment = self.segments.swap_remove(i);
                    start_open = false;
                } else if end_open && new_segment.end() == other.start() {
                    std::mem::swap(&mut other.points, &mut new_segment.points);
                    other.points.extend_from_slice(&new_segment.points[1..]);
                    new_segment = self.segments.swap_remove(i);
                    end_open = false;
                } else {
                    i += 1;
                }
            }
            // The segment can't link to any further segments. Add it to list.
            self.segments.push(new_segment);
        }
    }

    fn push_path(&mut self, segment: PathSegment) {
        self.segments.push(segment);
    }

    fn to_draw_commands(&self) -> impl '_ + Iterator<Item = DrawCommand> {
        self.segments.iter().flat_map(PathSegment::to_draw_commands)
    }
}

#[derive(Clone, Debug)]
pub struct ActivePath {
    style_id: u32,
    segment: PathSegment,
}

impl ActivePath {
    fn new() -> Self {
        Self {
            style_id: 0,
            segment: PathSegment::new(swf::Point::ZERO),
        }
    }

    fn add_point(&mut self, point: Point) {
        self.segment.add_point(point)
    }

    fn flush_fill(&mut self, start: swf::Point<Twips>, pending: &mut [PendingPath], flip: bool) {
        if self.style_id > 0 && !self.segment.is_empty() {
            if flip {
                self.segment.flip();
            }
            pending[self.style_id as usize - 1].add_segment(self.segment.clone());
        }
        self.segment.reset(start);
    }

    fn flush_stroke(&mut self, start: swf::Point<Twips>, pending: &mut [PendingPath]) {
        if self.style_id > 0 && !self.segment.is_empty() {
            pending[self.style_id as usize - 1].push_path(self.segment.clone());
        }
        self.segment.reset(start);
    }
}

pub struct ShapeConverter<'a> {
    // SWF shape commands.
    iter: std::slice::Iter<'a, swf::ShapeRecord>,

    // Pen position.
    cursor: swf::Point<Twips>,

    // Fill styles and line styles.
    // These change from StyleChangeRecords, and a flush occurs when these change.
    fill_styles: &'a [swf::FillStyle],
    line_styles: &'a [swf::LineStyle],

    fill_style0: ActivePath,
    fill_style1: ActivePath,
    line_style: ActivePath,
    winding_rule: FillRule,

    // Paths. These get flushed for each new layer.
    fills: Vec<PendingPath>,
    strokes: Vec<PendingPath>,

    // Output.
    commands: Vec<DrawPath<'a>>,
}

impl<'a> ShapeConverter<'a> {
    const DEFAULT_CAPACITY: usize = 512;

    fn from_shape(shape: &'a swf::Shape) -> Self {
        ShapeConverter {
            iter: shape.shape.iter(),

            cursor: swf::Point::ZERO,

            fill_styles: &shape.styles.fill_styles,
            line_styles: &shape.styles.line_styles,

            fill_style0: ActivePath::new(),
            fill_style1: ActivePath::new(),
            line_style: ActivePath::new(),

            fills: vec![PendingPath::new(); shape.styles.fill_styles.len()],
            strokes: vec![PendingPath::new(); shape.styles.line_styles.len()],

            commands: Vec::with_capacity(Self::DEFAULT_CAPACITY),

            winding_rule: if shape.flags.contains(swf::ShapeFlag::NON_ZERO_WINDING_RULE) {
                FillRule::NonZero
            } else {
                FillRule::EvenOdd
            },
        }
    }

    fn into_commands(mut self) -> Vec<DrawPath<'a>> {
        // As u32 is okay because SWF has a max of 65536 fills (TODO: should be u16?)
        let mut num_fill_styles = self.fill_styles.len() as u32;
        let mut num_line_styles = self.line_styles.len() as u32;
        while let Some(record) = self.iter.next() {
            match record {
                ShapeRecord::StyleChange(style_change) => {
                    if let Some(move_to) = &style_change.move_to {
                        self.cursor = *move_to;
                        // We've lifted the pen, so we're starting a new path.
                        // Flush the previous path.
                        self.flush_paths();
                    }

                    if let Some(styles) = &style_change.new_styles {
                        // A new style list is also used to indicate a new drawing layer.
                        self.flush_layer();
                        self.fill_styles = &styles.fill_styles;
                        self.line_styles = &styles.line_styles;
                        self.fills
                            .resize_with(self.fill_styles.len(), PendingPath::new);
                        self.strokes
                            .resize_with(self.line_styles.len(), PendingPath::new);
                        num_fill_styles = self.fill_styles.len() as u32;
                        num_line_styles = self.line_styles.len() as u32;
                    }

                    if let Some(new_style_id) = style_change.fill_style_1 {
                        self.fill_style1
                            .flush_fill(self.cursor, &mut self.fills, false);
                        // Validate in case we index an invalid fill style.
                        // <= because fill ID 0 (no fill) is implicit, so the array is actually 1-based
                        self.fill_style1.style_id = if new_style_id <= num_fill_styles {
                            new_style_id
                        } else {
                            0
                        };
                    }

                    if let Some(new_style_id) = style_change.fill_style_0 {
                        self.fill_style0
                            .flush_fill(self.cursor, &mut self.fills, true);
                        self.fill_style0.style_id = if new_style_id <= num_fill_styles {
                            new_style_id
                        } else {
                            0
                        }
                    }

                    if let Some(new_style_id) = style_change.line_style {
                        self.line_style.flush_stroke(self.cursor, &mut self.strokes);
                        self.line_style.style_id = if new_style_id <= num_line_styles {
                            new_style_id
                        } else {
                            0
                        }
                    }
                }
                ShapeRecord::StraightEdge { delta } => {
                    self.cursor += *delta;
                    self.visit_point(false);
                }
                ShapeRecord::CurvedEdge {
                    control_delta,
                    anchor_delta,
                } => {
                    self.cursor += *control_delta;
                    self.visit_point(true);

                    self.cursor += *anchor_delta;
                    self.visit_point(false);
                }
            }
        }

        // Flush any open paths.
        self.flush_layer();
        self.commands
    }

    /// Adds a point to the current path for the active fills/strokes.
    fn visit_point(&mut self, is_bezier_control: bool) {
        let point = Point {
            x: self.cursor.x,
            y: self.cursor.y,
            is_bezier_control,
        };
        if self.fill_style1.style_id > 0 {
            self.fill_style1.add_point(point);
        }
        if self.fill_style0.style_id > 0 {
            self.fill_style0.add_point(point);
        }
        if self.line_style.style_id > 0 {
            self.line_style.add_point(point);
        }
    }

    /// When the pen jumps to a new position, we reset the active path.
    fn flush_paths(&mut self) {
        // Move the current paths to the active list.
        self.fill_style1
            .flush_fill(self.cursor, &mut self.fills, false);
        self.fill_style0
            .flush_fill(self.cursor, &mut self.fills, true);
        self.line_style.flush_stroke(self.cursor, &mut self.strokes);
    }

    /// When a new layer starts, all paths are flushed and turned into drawing commands.
    fn flush_layer(&mut self) {
        self.flush_paths();

        // Draw fills, and then strokes.
        // Paths are drawn in order of style id, not based on the order of the draw commands.
        for (i, path) in self.fills.iter_mut().enumerate() {
            // These invariants are checked above (any invalid/empty fill ID should not have been added).
            debug_assert!(i < self.fill_styles.len());
            if path.segments.is_empty() {
                continue;
            }
            let style = unsafe { self.fill_styles.get_unchecked(i) };
            self.commands.push(DrawPath::Fill {
                style,
                commands: path.to_draw_commands().collect(),
                winding_rule: self.winding_rule,
            });
            path.segments.clear();
        }

        // Strokes are drawn last because they always appear on top of fills in the same layer.
        // Because path segments can either be open or closed, we convert each stroke segment into
        // a separate draw command.
        for (i, path) in self.strokes.iter_mut().enumerate() {
            debug_assert!(i < self.line_styles.len());
            let style = unsafe { self.line_styles.get_unchecked(i) };
            for segment in &path.segments {
                if segment.is_empty() {
                    continue;
                }
                self.commands.push(DrawPath::Stroke {
                    style,
                    is_closed: segment.is_closed(),
                    commands: segment.to_draw_commands().collect(),
                });
            }
            path.segments.clear();
        }
    }
}
