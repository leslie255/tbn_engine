pub trait Bindable {
    fn bind_group_layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry;
    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry;
}

pub trait AsBindGroup {
    fn bind_group_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry>;
    fn bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry>;
}

/// TODO: Make this into a derive macro so it supports structs with generic parameters.
#[macro_export]
macro_rules! impl_as_bind_group {
    ($T:path { $($binding_id:literal => $field:ident),* $(,)? } $($tts:tt)*) => {
        impl $crate::AsBindGroup for $T {
            fn bind_group_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
                ::std::vec![
                    $($crate::Bindable::bind_group_layout_entry(
                        &self.$field,
                        $binding_id,
                    )),*
                ]
            }
            fn bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
                ::std::vec![
                    $($crate::Bindable::bind_group_entry(
                        &self.$field,
                        $binding_id,
                    )),*
                ]
            }
        }
        $crate::impl_as_bind_group! { $($tts)* }
    };
    () => {}
}

pub(crate) fn create_wgpu_bind_group_layout(
    device: &wgpu::Device,
    bind_group: &impl AsBindGroup,
) -> wgpu::BindGroupLayout {
    let label = Some(std::any::type_name_of_val(&bind_group));
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label,
        entries: &bind_group.bind_group_layout_entries(),
    })
}

pub(crate) fn create_wgpu_bind_group(
    device: &wgpu::Device,
    bind_group: &impl AsBindGroup,
) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    let label = Some(std::any::type_name_of_val(&bind_group));
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label,
        entries: &bind_group.bind_group_layout_entries(),
    });
    let wgpu_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label,
        layout: &layout,
        entries: &bind_group.bind_group_entries(),
    });
    (wgpu_bind_group, layout)
}

