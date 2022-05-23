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
            &mut $modelv:ident,
            $widgetsv:ident,
            $messagev:ident,
            $inputv2:ident,
            $outputv2:ident
        ) $update:block

        $(#[$attr6:meta])*
        async fn command($cmdv:ident: $cmd:ty, $inputv3:ident) { $($cmd_expr:expr)* }
    ) => {
        $(#[$attr])*
        $mvis struct $model {
            $($mpvis $property: $type,)*
        }

        $(#[$attr2])*
        $wvis struct $widgets_ {
            $($wpvis $widgets_property: $widgets_type,)*
        }

        impl StatefulComponent for $model {
            type Payload = $args;
            type Input = $input;
            type Output = $output;
            type Root = $root;
            type Widgets = $widgets_;
            type Command = $cmd;

            fn init_root() -> Self::Root $init_root

            $(#[$attr4])*
            fn dock(
                $argsv: Self::Payload,
                $rootv: &Self::Root,
                $inputv: &mut Sender<Self::Input>,
                $outputv: &mut Sender<Self::Output>
            ) -> Fuselage<Self, Self::Widgets> $init_view

            $(#[$attr5])*
            fn update(
                &mut $modelv,
                $widgetsv: &mut Self::Widgets,
                $messagev: Self::Input,
                $inputv2: &mut Sender<Self::Input>,
                $outputv2: &mut Sender<Self::Output>
            ) -> Option<Self::Command> $update

            $(#[$attr6])*
            fn command($cmdv: $cmd, $inputv3: Sender<Self::Input>) -> CommandFuture {
                Box::pin(async move { $($cmd_expr)* } )
            }
        }
    }
}

#[macro_export]
macro_rules! elm_component {
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
            &mut $modelv:ident,
            $messagev:ident,
            $inputv2:ident,
            $outputv2:ident
        ) $update:block

        $(#[$attr6:meta])*
        fn update_view(
            &$modelv2:ident,
            $widgetsv:ident,
            $inputv3:ident,
            $outputv3:ident
        ) $update_view:block

        $(#[$attr7:meta])*
        async fn command($cmdv:ident: $cmd:ty, $inputv4:ident) { $($cmd_expr:expr)* }
    ) => {
        $(#[$attr])*
        $mvis struct $model {
            $($mpvis $property: $type,)*
        }

        $(#[$attr2])*
        $wvis struct $widgets_ {
            $($wpvis $widgets_property: $widgets_type,)*
        }

        impl StatefulComponent for $model {
            type Payload = $args;
            type Input = $input;
            type Output = $output;
            type Root = $root;
            type Widgets = $widgets_;
            type Command = $cmd;

            fn init_root() -> Self::Root $init_root

            $(#[$attr4])*
            fn dock(
                $argsv: Self::Payload,
                $rootv: &Self::Root,
                $inputv: &mut Sender<Self::Input>,
                $outputv: &mut Sender<Self::Output>
            ) -> Fuselage<Self, Self::Widgets> $init_view

            $(#[$attr5])*
            fn update(
                &mut $modelv,
                $messagev: Self::Input,
                $inputv2: &mut Sender<Self::Input>,
                $outputv2: &mut Sender<Self::Output>
            ) -> Option<Self::Command> $update

            $(#[$attr6])*
            fn update_view(
                &$modelv2,
                $widgetsv: &mut Self::Widgets,
                $inputv3: &mut Sender<Self::Input>,
                $outputsv3: &mut Sender<Self::Output>
            ) $update_view

            $(#[$attr7])*
            fn command($cmdv: $cmd, $inputv4: Sender<Self::Input>) -> CommandFuture {
                Box::pin(async move { $($cmd_expr)* } )
            }
        }
    }
}
