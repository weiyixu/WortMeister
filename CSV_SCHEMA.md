# 可扩展 CSV 存储格式

```text
id,level,lesson,german,chinese,example,example_zh,tags,enabled
```

- `id`: 永久唯一键。新增词条时创建新 ID，不要修改旧 ID，否则学习进度无法关联。
- `level`: 级别，例如 A1.1、A1.2、A2.1。
- `lesson`: 课次，可自由扩展。
- `german`: 德语答案。名词建议带冠词。
- `chinese`: 中文释义。
- `example`: 德语例句。
- `example_zh`: 例句中文翻译。
- `tags`: 使用英文分号分隔，例如 `work;radar;validation`。
- `enabled`: `true` 或 `false`。设为 false 可暂时隐藏词条。

文件必须保存为 UTF-8 CSV。Excel 打开后请使用“CSV UTF-8（逗号分隔）”格式保存。
学习进度单独保存在 `progress.json`，因此替换或扩充词库不会覆盖学习记录。
