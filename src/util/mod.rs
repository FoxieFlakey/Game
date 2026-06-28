pub mod identifier;
pub mod sig_safe;

macro_rules! static_gpu_buffer {
    ($($visibility:vis static $kind:ident $name:ident : LazyLock<VecBuf<[ $item_ty:ty ]>> => $init:expr ; )* ) => {
        $(
            $visibility static $name: std::sync::LazyLock<$crate::rendering::buffer::VecBuf<$item_ty>> = std::sync::LazyLock::new(|| {
                let entries: [$item_ty; _] = $init;
                let device = $crate::states::main_dev::get().clone();
                let mut buf = $crate::rendering::buffer::VecBuf::new_with_initial_capacity(
                    device,
                    $crate::rendering::buffer::BufferKind::$kind,
                    entries.len()
                );
                buf.extend_from_slice($crate::states::data_loader::get(), &entries);

                buf
            });
        )*
    }
}
pub(crate) use static_gpu_buffer;

macro_rules! vec_buf {
    ( $device:expr, $data_loader:expr, $kind:ident, $init:expr ) => {{
        let entries: [_; _] = $init;
        let device: wgpu::Device = $device;
        let data_loader: &$crate::rendering::data_loader::DataLoader = $data_loader;
        let mut buf = $crate::rendering::buffer::VecBuf::new_with_initial_capacity(
            device,
            $crate::rendering::buffer::BufferKind::$kind,
            entries.len(),
        );
        buf.extend_from_slice(data_loader, &entries);

        buf
    }};
}
pub(crate) use vec_buf;

macro_rules! vec_buf2 {
    ( $kind:ident, $init:expr ) => {{
        let entries: [_; _] = $init;
        let device: wgpu::Device = $crate::states::main_dev::get().clone();
        let data_loader: &$crate::rendering::data_loader::DataLoader =
            $crate::states::data_loader::get();
        let mut buf = $crate::rendering::buffer::VecBuf::new_with_initial_capacity(
            device,
            $crate::rendering::buffer::BufferKind::$kind,
            entries.len(),
        );
        buf.extend_from_slice(data_loader, &entries);

        buf
    }};
}
pub(crate) use vec_buf2;
