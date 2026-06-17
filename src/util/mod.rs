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
