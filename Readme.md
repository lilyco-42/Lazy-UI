# 如你所见 这是一个基于[plyx](https://github.com/TheRedDeveloper/ply-engine/)的简单上层封装
我们的目的是实现这样的ui书写

```
 侧边栏({
        启动()
        设置()
        关于()
        作者()
    })
    日志面板()
    日志进度条(默认 = nvim dialog 样式)
```
- 思路: 约定大于配置.我们约定 
 每一个组件 Button.rs 对应一个Button.toml 
    - *.rs文件
        负责数据处理和响应操作 
    - *.toml
        实现颜色,ui缩放等不影响原有功能的实现
- 为了易用性,我们确保
    1.全局config.toml 必须实现跨平台ui渲染一致,必须要要有缩放按钮.
    2.每个组件开发者应提供良好的布局 
    - 比如开发者1 开发Button.rs 和 Button.toml,Button.toml 必须是该作者默认设置的最优显示,在toml内标注适用的系统分辨率等.
-
