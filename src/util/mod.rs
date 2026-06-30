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

// NOTE this does not make the T implements Default
pub const trait ConstDefault: Sized {
    const DEFAULT: Self;
    
    fn const_default() -> Self {
        Self::DEFAULT
    }
}

macro_rules! impl_const_default {
    ($type:ty, $default:expr) => {
        impl $crate::util::ConstDefault for $type {
            const DEFAULT: Self = $default;
        }
        
        impl std::default::Default for $type {
            fn default() -> Self {
                <Self as $crate::util::ConstDefault>::const_default()
            }
        }
    };
}
pub(crate) use impl_const_default;

/// Example, its like
/// 
/// taffy::Style {
///   ..Default::default()
/// }
/// 
/// But works in const context, it cause some error..
/// 
/// # Example
/// ```rust
/// taffy_style! {
///     size: taffy::Size {
///         width: taffy::Dimension::percent(1.0),
///         height: taffy::Dimension::percent(1.0)
///     },
///     // The rest is using taffy::Style::DEFAULT values
/// }
/// ```
macro_rules! taffy_style {
    ($($field:ident : $val:expr),* $(,)?) => {{
        let mut s = taffy::Style::DEFAULT;
        $( s.$field = $val; )*
        s
    }};
}
pub(crate) use taffy_style;

