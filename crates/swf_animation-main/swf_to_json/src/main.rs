use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    env,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use animation::generation_animation;
use bitmap::CompressedBitmap;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use render::{
    get_device_and_queue,
    mesh::{GradientUniform, VertexColor, VertexPosition},
};
use ruffle_render::tessellator::{DrawType, Gradient};
use swf::{CharacterId, DefineBitsLossless, GradientInterpolation, SwfStr};
use tessellator::ShapeTessellator;
use tracing::{error, info};
use tracing_subscriber::{
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use wgpu::{VertexBufferLayout, util::DeviceExt};
mod animation;
pub mod bitmap;
mod render;
mod tessellator;

/// 制作多大得渐变纹理，
const GRADIENT_SIZE: usize = 256;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewMatrix {
    pub matrix: [[f32; 4]; 4],
}

impl ViewMatrix {
    fn bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        width: f32,
        height: f32,
    ) -> wgpu::BindGroup {
        let view_matrix = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("View Matrix Buffer"),
            contents: bytemuck::cast_slice(&[view_matrix]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("View Matrix Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextureTransform {
    pub texture_matrix: [[f32; 4]; 4],
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// 输入的swf文件名
    file_path: String,
    /// 图片放大倍数，默认为1
    #[arg(short, long, default_value = "1.0")]
    scale: f32,
    /// 输出的目录
    output: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file_path = &args.file_path;

    let env_filter = tracing_subscriber::EnvFilter::builder().parse_lossy(
        env::var("RUST_LOG")
            .as_deref()
            .unwrap_or("error,convert=info"),
    );
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();

    let reader = BufReader::new(File::open(file_path)?);
    let swf_buf = swf::decompress_swf(reader)?;
    let swf: swf::Swf<'_> = swf::parse_swf(&swf_buf)?;
    let tags = swf.tags;

    let mut bitmaps = HashMap::new();
    for tag in &tags {
        match tag {
            swf::Tag::DefineBitsJpeg3(jpeg_data) => {
                let (width, height) =
                    ruffle_render::utils::decode_define_bits_jpeg_dimensions(jpeg_data.data)
                        .unwrap();

                bitmaps.insert(
                    jpeg_data.id,
                    CompressedBitmap::Jpeg {
                        data: jpeg_data.data.to_vec(),
                        alpha: Some(jpeg_data.alpha_data.to_vec()),
                        width,
                        height,
                    },
                );
            }
            swf::Tag::DefineBitsLossless(bit_loss_less) => {
                bitmaps.insert(
                    bit_loss_less.id,
                    CompressedBitmap::Lossless(DefineBitsLossless {
                        version: bit_loss_less.version,
                        id: bit_loss_less.id,
                        format: bit_loss_less.format,
                        width: bit_loss_less.width,
                        height: bit_loss_less.height,
                        data: Cow::Owned(bit_loss_less.data.clone().into_owned()),
                    }),
                );
            }
            _ => {
                continue;
            }
        }
    }

    // 获取Graphic Tag
    let shapes: Vec<_> = tags
        .iter()
        .filter(|tag| matches!(tag, swf::Tag::DefineShape(_)))
        .map(|tag| {
            if let swf::Tag::DefineShape(shape) = tag {
                shape
            } else {
                unreachable!()
            }
        })
        .collect();

    // 解析出文件名
    let path = Path::new(file_path);
    let file_name = if let Some(file_name) = path.file_name() {
        file_name.to_str().unwrap()
    } else {
        return Err(anyhow::anyhow!("无法获取文件名"));
    };

    let output = if let Some(output) = &args.output {
        let path = Path::new(output);
        path.to_path_buf()
    } else {
        // 默认使用当前文件夹路径
        // 获取当前文件路径
        let current_dir: PathBuf = env::current_dir()?;
        let file_name: Vec<&str> = file_name.split(".").collect();
        current_dir.join("output").join(file_name.first().unwrap())
    };
    // 判断路径是否存在，如果不存在则创建
    if !output.exists() {
        std::fs::create_dir_all(&output)?;
    }
    let encoding_for_version = SwfStr::encoding_for_version(swf.header.version());

    let mut shape_transform = BTreeMap::new();
    generation_shape_image(
        args.scale as f64,
        shapes,
        &mut shape_transform,
        bitmaps,
        &output,
    )?;
    generation_animation(
        tags,
        shape_transform,
        file_name,
        &output,
        swf.header.frame_rate().to_f32() as u16,
        encoding_for_version,
    )?;
    Ok(())
}

fn generation_shape_image(
    scale: f64,
    shapes: Vec<&swf::Shape>,
    shape_transform: &mut BTreeMap<CharacterId, (f32, f32)>,
    bitmaps: HashMap<CharacterId, CompressedBitmap>,
    output: &Path,
) -> anyhow::Result<()> {
    info!("开始绘制图形, 放大倍数: {}", scale);
    // 配置渲染器
    let (device, queue) = get_device_and_queue()?;
    let (
        (color_render_pipeline, view_matrix_bind_group_layout),
        (gradient_render_pipeline, gradient_bind_group_layout),
        (bitmap_render_pipeline, bitmap_bind_group_layout),
    ) = create_render_pipelines(&device);

    let pb = ProgressBar::new(shapes.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    // 解析Shape渲染
    let mut tessellator = ShapeTessellator::new();
    for shape in shapes {
        pb.inc(1);
        let x_min = shape.shape_bounds.x_min.to_pixels();
        let y_min = shape.shape_bounds.y_min.to_pixels();
        let x_max = shape.shape_bounds.x_max.to_pixels();
        let y_max = shape.shape_bounds.y_max.to_pixels();
        shape_transform.insert(
            shape.id,
            ((x_min + x_max) as f32 / 2.0, (y_min + y_max) as f32 / 2.0),
        );
        // 计算(x_min, y_min)的到(0,0)的偏移量
        let x_offset = 0.0 - x_min as f32;
        let y_offset = 0.0 - y_min as f32;

        let mut width = shape.shape_bounds.width().to_pixels();
        let mut height = shape.shape_bounds.height().to_pixels();

        width *= scale;
        height *= scale;

        let size = wgpu::Extent3d {
            width: width as u32 + 1,
            height: height as u32 + 1,
            depth_or_array_layers: 1,
        };

        let msaa_texture = create_texture(
            &device,
            Some("多重采样抗锯齿纹理"),
            size,
            4,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let output_texture = create_texture(
            &device,
            Some("输出纹理"),
            size,
            1,
            wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let lyon_mesh = tessellator.tessellate_shape(shape.into(), &bitmaps);

        let mut gradient_textures = Vec::new();
        let gradients = lyon_mesh.gradients;
        for gradient in gradients {
            generation_gradient(&mut gradient_textures, gradient, &device, &queue);
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Color Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &msaa_view,
                resolve_target: Some(&output_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        let view_matrix_bind_group = ViewMatrix::bind_group(
            &device,
            &view_matrix_bind_group_layout,
            (width / scale) as f32,
            (height / scale) as f32,
        );
        for draw in lyon_mesh.draws {
            let indices = draw.indices;
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("索引缓冲区"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
            match draw.draw_type {
                DrawType::Color => {
                    let mut meshes = Vec::with_capacity(draw.vertices.len());
                    for vertex in draw.vertices {
                        let mesh = VertexColor::new(
                            [vertex.x + x_offset, vertex.y + y_offset],
                            [
                                vertex.color.r as f32 / 255.0,
                                vertex.color.g as f32 / 255.0,
                                vertex.color.b as f32 / 255.0,
                                vertex.color.a as f32 / 255.0,
                            ],
                        );
                        meshes.push(mesh);
                    }

                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("顶点缓冲区"),
                            contents: bytemuck::cast_slice(&meshes),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    render_pass.set_pipeline(&color_render_pipeline);
                    render_pass.set_bind_group(0, &view_matrix_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                }
                DrawType::Gradient { matrix, gradient } => {
                    let mut meshes = Vec::with_capacity(draw.vertices.len());
                    for vertex in draw.vertices {
                        let mesh = VertexPosition::new([vertex.x, vertex.y]);
                        meshes.push(mesh);
                    }
                    let transform = [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [x_offset, y_offset, 0.0, 1.0],
                    ];

                    let transform_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("变换缓冲区"),
                            contents: bytemuck::cast_slice(&[transform]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                    let gradient_texture = gradient_textures.get(gradient).unwrap();
                    let gradient_uniform = GradientUniform {
                        ..gradient_texture.1
                    };
                    // 将matrix 转为[f32; 4; 4]
                    let mut texture_transform = [[0.0; 4]; 4];
                    texture_transform[0][..3].copy_from_slice(&matrix[0]);
                    texture_transform[1][..3].copy_from_slice(&matrix[1]);
                    texture_transform[2][..3].copy_from_slice(&matrix[2]);

                    let texture_transform_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("纹理变换缓冲区"),
                            contents: bytemuck::cast_slice(&[texture_transform]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                    let gradient_uniform_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("渐变缓冲区"),
                            contents: bytemuck::cast_slice(&[gradient_uniform]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &gradient_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&gradient_texture.0),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: texture_transform_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: gradient_uniform_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: transform_buffer.as_entire_binding(),
                            },
                        ],
                        label: Some("gradient texture bind group"),
                    });

                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("顶点缓冲区"),
                            contents: bytemuck::cast_slice(&meshes),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    render_pass.set_pipeline(&gradient_render_pipeline);
                    render_pass.set_bind_group(0, &view_matrix_bind_group, &[]);
                    render_pass.set_bind_group(1, &bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                }
                DrawType::Bitmap(bitmap) => {
                    let mut meshes = Vec::with_capacity(draw.vertices.len());
                    for vertex in draw.vertices {
                        let mesh = VertexPosition::new([vertex.x, vertex.y]);
                        meshes.push(mesh);
                    }

                    let transform = [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [x_offset, y_offset, 0.0, 1.0],
                    ];

                    let transform_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("transform buffer"),
                            contents: bytemuck::cast_slice(&transform),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("顶点缓冲区"),
                            contents: bytemuck::cast_slice(&meshes),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                    let mut texture_transform = [[0.0; 4]; 4];
                    texture_transform[0][..3].copy_from_slice(&bitmap.matrix[0]);
                    texture_transform[1][..3].copy_from_slice(&bitmap.matrix[1]);
                    texture_transform[2][..3].copy_from_slice(&bitmap.matrix[2]);
                    if let Some(compressed_bitmap) = bitmaps.get(&bitmap.bitmap_id) {
                        let decoded = match compressed_bitmap.decode() {
                            Ok(decoded) => decoded,
                            Err(e) => {
                                error!("Failed to decode bitmap: {:?}", e);
                                continue;
                            }
                        };
                        let bitmap = decoded.to_rgba();
                        let texture = device.create_texture_with_data(
                            &queue,
                            &wgpu::TextureDescriptor {
                                label: Some("Bitmap Texture"),
                                size: wgpu::Extent3d {
                                    width: bitmap.width(),
                                    height: bitmap.height(),
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                                usage: wgpu::TextureUsages::TEXTURE_BINDING
                                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                                view_formats: &[],
                            },
                            wgpu::util::TextureDataOrder::LayerMajor,
                            bitmap.data(),
                        );

                        let texture_transform_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Texture Transform Buffer"),
                                contents: bytemuck::cast_slice(&texture_transform),
                                usage: wgpu::BufferUsages::UNIFORM,
                            });

                        let texture_view =
                            texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

                        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Bitmap Bind Group"),
                            layout: &bitmap_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&texture_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&sampler),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: texture_transform_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: transform_buffer.as_entire_binding(),
                                },
                            ],
                        });
                        render_pass.set_pipeline(&bitmap_render_pipeline);
                        render_pass.set_bind_group(0, &view_matrix_bind_group, &[]);
                        render_pass.set_bind_group(1, &bind_group, &[]);
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..indices.len() as _, 0, 0..1);
                    }
                }
            }
        }
        drop(render_pass);
        queue.submit(std::iter::once(encoder.finish()));
        let unpadded_byte_per_row = size.width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_byte_per_row_padding = (align - unpadded_byte_per_row % align) % align;
        let padded_byte_per_row = unpadded_byte_per_row + padded_byte_per_row_padding;
        // 将输出纹理提取到缓冲区中
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: (padded_byte_per_row * size.height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_byte_per_row),
                    rows_per_image: None,
                },
            },
            size,
        );
        // 从缓冲区数据到image
        queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        let _ = device.poll(wgpu::PollType::Wait);
        let _ = receiver.recv().expect("MPSC channel must not fail");
        let data = buffer_slice.get_mapped_range();

        let mut bytes = Vec::with_capacity(size.height as usize);
        for pixel in data.chunks_exact(padded_byte_per_row as usize) {
            bytes.extend_from_slice(&pixel[..unpadded_byte_per_row as usize]);
        }
        let image_buffer = image::RgbaImage::from_raw(size.width, size.height, bytes)
            .expect("Retrieved texture buffer must be a valid RgbaImage");
        image_buffer
            .save(output.join(format!("{}.png", shape.id)))
            .expect("Failed to save image");
    }

    pb.finish_with_message(format!("转换完成, 路径: {}", output.display()));
    Ok(())
}

fn generation_gradient(
    gradient_textures: &mut Vec<(wgpu::TextureView, GradientUniform)>,
    gradient: Gradient,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    let colors = if gradient.records.is_empty() {
        vec![0; GRADIENT_SIZE * 4]
    } else {
        let mut colors = vec![0; GRADIENT_SIZE * 4];
        let convert = if gradient.interpolation == GradientInterpolation::LinearRgb {
            |color| srgb_to_linear(color / 255.0) * 255.0
        } else {
            |color| color
        };

        for t in 0..GRADIENT_SIZE {
            let mut last = 0;
            let mut next = 0;
            for (i, record) in gradient.records.iter().enumerate().rev() {
                if (record.ratio as usize) < t {
                    last = i;
                    next = (i + 1).min(gradient.records.len() - 1);
                    break;
                }
            }
            assert!(last == next || last + 1 == next);
            let last_record = &gradient.records[last];
            let next_record = &gradient.records[next];
            let factor = if next == last {
                0.0
            } else {
                (t as f32 - last_record.ratio as f32)
                    / (next_record.ratio as f32 - last_record.ratio as f32)
            };

            colors[t * 4] = lerp(
                convert(last_record.color.r as f32),
                convert(next_record.color.r as f32),
                factor,
            ) as u8;
            colors[(t * 4) + 1] = lerp(
                convert(last_record.color.g as f32),
                convert(next_record.color.g as f32),
                factor,
            ) as u8;
            colors[(t * 4) + 2] = lerp(
                convert(last_record.color.b as f32),
                convert(next_record.color.b as f32),
                factor,
            ) as u8;
            colors[(t * 4) + 3] = lerp(
                last_record.color.a as f32,
                next_record.color.a as f32,
                factor,
            ) as u8;
        }
        colors
    };

    let size = wgpu::Extent3d {
        width: GRADIENT_SIZE as u32,
        height: 1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("Shape"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &colors[..],
    );
    let gradient_uniform = GradientUniform::from(gradient);
    gradient_textures.push((
        texture.create_view(&wgpu::TextureViewDescriptor::default()),
        gradient_uniform,
    ));
}

fn create_render_pipelines(
    device: &wgpu::Device,
) -> (
    (wgpu::RenderPipeline, wgpu::BindGroupLayout),
    (wgpu::RenderPipeline, wgpu::BindGroupLayout),
    (wgpu::RenderPipeline, wgpu::BindGroupLayout),
) {
    let color_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Color Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("render/shaders/color.wgsl").into()),
    });

    let view_matrix_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Color Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Color Render Pipeline Layout"),
        bind_group_layouts: &[&view_matrix_bind_group_layout],
        push_constant_ranges: &[],
    });

    let color_render_pipeline = create_render_pipeline(
        device,
        Some("Color Render Pipeline"),
        &color_shader,
        &render_pipeline_layout,
        VertexColor::desc(),
    );

    let gradient_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Gradient Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("render/shaders/gradient.wgsl").into()),
    });

    let gradient_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Gradient Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Gradient Render Pipeline Layout"),
        bind_group_layouts: &[&view_matrix_bind_group_layout, &gradient_bind_group_layout],
        push_constant_ranges: &[],
    });

    let gradient_render_pipeline = create_render_pipeline(
        device,
        Some("Gradient Render Pipeline"),
        &gradient_shader,
        &render_pipeline_layout,
        VertexPosition::desc(),
    );

    let bitmap_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Bitmap Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("render/shaders/bitmap.wgsl").into()),
    });

    let bitmap_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bitmap Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Bitmap Render Pipeline Layout"),
        bind_group_layouts: &[&view_matrix_bind_group_layout, &bitmap_bind_group_layout],
        push_constant_ranges: &[],
    });
    let bitmap_render_pipeline = create_render_pipeline(
        device,
        Some("Bitmap Render Pipeline"),
        &bitmap_shader,
        &render_pipeline_layout,
        VertexPosition::desc(),
    );

    (
        (color_render_pipeline, view_matrix_bind_group_layout),
        (gradient_render_pipeline, gradient_bind_group_layout),
        (bitmap_render_pipeline, bitmap_bind_group_layout),
    )
}

fn create_render_pipeline(
    device: &wgpu::Device,
    label: Option<&str>,
    shader: &wgpu::ShaderModule,
    render_pipeline_layout: &wgpu::PipelineLayout,
    buffer: VertexBufferLayout,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[buffer],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fragment"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    })
}

fn create_texture(
    device: &wgpu::Device,
    label: Option<&str>,
    size: wgpu::Extent3d,
    sample_count: u32,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage,
        view_formats: &[],
    })
}

/// Converts an RGBA color from sRGB space to linear color space.
fn srgb_to_linear(color: f32) -> f32 {
    if color <= 0.04045 {
        color / 12.92
    } else {
        f32::powf((color + 0.055) / 1.055, 2.4)
    }
}

/// 线性插值
fn lerp(a: f32, b: f32, factor: f32) -> f32 {
    a + (b - a) * factor
}
