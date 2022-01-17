# 内存模型

## 地址空间

| 类型    | 地址范围 | 长度 |
| ------ | -------- | ---- |
| Flash | 0x08000000 - 0x0807FFFF | 512K |
| SRAM | 0x20000000 - 0x2000FFFF | 64K |

## Flash 地址分配

| 类型 | 地址范围 | 长度 |
| --- | --- | --- |
| 引导程序 | 0x08000000 - 0x08004000 | 16K |
| 操作系统 | 0x08004000 - 0x08014000 | 64K |
| 用户程序 | 0x08014000 - 0x08054000 | 256K |
| 保留 | 0x08054000 - 0x0007FFFF | 176K |

## 引导程序入口点

调试：0x20000000 (SRAM)  
生产：0x08000000 (Flash)

## 内存地址分配

### 引导程序层

| 类型 | 地址范围 | 长度 |
| --- | --- | --- |
| 引导程序 | 0x20000000 - 0x20000500 | 1280B |
| 其他 | 0x20000500 - 0x2000F500 | 61440B |
| 保留 | 0x2000F500 - 0x2000FFFF | 2815B |

### 操作系统层

