with open("src/locales/zh.ts", "r", encoding="utf-8") as f:
    zh = f.read()

old = '  "config.max_body_size.desc": "最大请求体大小（字节），默认 1 GB。填 0 为无限。重启桥接生效。",'
new = '  "config.max_body_size.desc": "最大请求体大小（MB），默认 1024。填 0 为无限。重启桥接生效。",'

print("old found:", old in zh)
if old in zh:
    zh = zh.replace(old, new)

old2 = '  "config.max_body_size.placeholder": "字节（如 1073741824）",'
new2 = '  "config.max_body_size.unit": "MB",'
print("old2 found:", old2 in zh)
if old2 in zh:
    zh = zh.replace(old2, new2)

with open("src/locales/zh.ts", "w", encoding="utf-8") as f:
    f.write(zh)
print("Done")
