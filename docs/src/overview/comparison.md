# 与其他引擎的比较
常用指令
```
find . -not -name ".DS_Store" -print | sed -e 's;[^/]*/;|____;g;s;____|; |;g'
```