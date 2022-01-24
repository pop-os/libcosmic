// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#[macro_export]
macro_rules! component {
    (
        $(#[$attr:meta])*
        $mvis:vis struct $model:ident ($args:ty) {
            $(
                $mpvis:vis $property:ident : $type:ty,
            )*
        }

        $(#[$attr2:meta])*
        $wvis:vis struct $widgets_:ident($root:ty) {
            $(
                $wpvis:vis $widgets_property:ident : $widgets_type:ty,
            )*
        }

        type Input = $input:ty;
        type Output = $output:ty;

        $(#[$attr3:meta])*
        fn init_view(
            $selfv:ident,
            $argsv:ident,
            $inputv:ident,
            $outputv:ident
        ) $init_view:block

        $(#[$attr4:meta])*
        fn update(
            $selfv2:ident,
            $widgetsv:ident,
            $messagev:ident,
            $inputv2:ident,
            $outputv2:ident
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
            type InitialArgs = $args;
            type Input = $input;
            type Output = $output;
            type RootWidget = $root;
            type Widgets = $widgets_;

            $(#[$attr3])*
            fn init_view(
                &mut $selfv2,
                $argsv: Self::InitialArgs,
                $inputv: &mut Sender<Self::Input>,
                $outputv: &mut Sender<Self::Output>
            ) -> (Self::Widgets, Self::RootWidget) $init_view

            $(#[$attr4])*
            fn update(
                &mut $selfv2,
                $widgetsv: &mut Self::Widgets,
                $messagev: Self::Input,
                $inputv2: &mut Sender<Self::Input>,
                $outputv2: &mut Sender<Self::Output>
            ) $update
        }
    }
}
