fn main() {
    // 只在原生 Windows 构建时嵌入图标与版本信息，交叉编译时由 CI 在 Windows runner 上处理。
    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("../../resources/icons/icon.ico");
        resource.set("ProductName", "BiuViewer");
        resource.set("FileDescription", "博览 BiuViewer");
        resource.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        resource.set("FileVersion", env!("CARGO_PKG_VERSION"));
        if let Err(err) = resource.compile() {
            println!("cargo:warning=嵌入 Windows 图标失败: {err}");
        }
    }
}
