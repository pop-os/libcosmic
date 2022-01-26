// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#[macro_export]
macro_rules! component {
    (
        $(#[$attr:meta])*
        $mvis:vis struct $model:ident {
            $(
                $mpvis:vis $property:ident : $type:ty,
            )*
        }

        $(#[$attr2:meta])*
        $wvis:vis struct $widgets_:ident {
            $(
                $wpvis:vis $widgets_property:ident : $widgets_type:ty,
            )*
        }

        type Input = $input:ty;
        type Output = $output:ty;
        type Root = $root:ty $init_root:block;

        $(#[$attr4:meta])*
        fn init(
            $argsv:ident: $args:ty,
            $rootv:ident,
            $inputv:ident,
            $outputv:ident
        ) $init_view:block

        $(#[$attr5:meta])*
        fn update(
            $componentv:ident,
            $messagev:ident
        ) $update:block
    ) => {
        $(#[$attr])*
        $mvis struct $model {
            $($mpvis $property: $type,)*
        }

        $(#[$attr2])*
        $wvis struct $widgets_ {
            $($wpvis $widgets_property: $widgets_type,)*
        }

        impl Component for $model {
            type InitParams = $args;
            type Input = $input;
            type Output = $output;
            type Root = $root;
            type Widgets = $widgets_;

            fn init_root() -> Self::Root $init_root

            $(#[$attr4])*
            fn init_inner(
                $argsv: Self::InitParams,
                $rootv: &Self::Root,
                $inputv: Sender<Self::Input>,
                $outputv: Sender<Self::Output>
            ) -> ComponentInner<Self, Self::Widgets, Self::Input, Self::Output> $init_view

            $(#[$attr5])*
            fn update(
                $componentv: &mut ComponentInner<Self, Self::Widgets, Self::Input, Self::Output>,
                $messagev: Self::Input,
            ) $update
        }
    }
}
