# 进程调度

最大进程数: 12  
进程调度策略: 带优先级抢占的时间片轮转调度策略

## PCB 

PCB 总大小为 

| 属性 | 类型 | 说明 | 
| ---- | ---- | ---- |
| pid | u16 | 进程号 |
| ppid | u16 | 父进程号 |
| priority | u16 | 优先级 |
| system | bool | 是否为系统进程 |
| state | u8 | 状态 |
| start_time | u32 | 启动时间 |
| time_span_used | u32 | 时间片使用数 |


## 进程调度流程

见 [context_switch.dot](./context_switch.dot)



