use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Soft,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonSize {
    XS,
    SM,
    MD,
    LG,
    XL,
}

impl ButtonSize {
    fn classes(&self) -> &'static str {
        match self {
            ButtonSize::XS => "rounded-sm px-2 py-1 text-xs",
            ButtonSize::SM => "rounded-sm px-2 py-1 text-sm",
            ButtonSize::MD => "rounded-md px-2.5 py-1.5 text-sm",
            ButtonSize::LG => "rounded-md px-3 py-2 text-sm",
            ButtonSize::XL => "rounded-md px-3.5 py-2.5 text-sm",
        }
    }

    fn gap_classes(&self) -> &'static str {
        match self {
            ButtonSize::XS | ButtonSize::SM | ButtonSize::MD | ButtonSize::LG => "gap-x-1.5",
            ButtonSize::XL => "gap-x-2",
        }
    }
}

impl ButtonVariant {
    fn classes(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "bg-indigo-600 text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 dark:bg-indigo-500 dark:shadow-none dark:hover:bg-indigo-400 dark:focus-visible:outline-indigo-500",
            ButtonVariant::Secondary => "bg-white text-gray-900 shadow-xs inset-ring inset-ring-gray-300 hover:bg-gray-50 dark:bg-white/10 dark:text-white dark:shadow-none dark:inset-ring-white/5 dark:hover:bg-white/20",
            ButtonVariant::Soft => "bg-indigo-50 text-indigo-600 shadow-xs hover:bg-indigo-100 dark:bg-indigo-500/20 dark:text-indigo-400 dark:shadow-none dark:hover:bg-indigo-500/30",
        }
    }
}

fn build_button_classes(
    variant: ButtonVariant,
    size: ButtonSize,
    has_icon: bool,
    loading: bool,
) -> String {
    let base = "font-semibold";
    let variant_classes = variant.classes();
    let size_classes = size.classes();
    let layout = if has_icon || loading {
        format!("inline-flex items-center {}", size.gap_classes())
    } else {
        String::new()
    };

    format!("{} {} {} {}", base, variant_classes, size_classes, layout)
        .trim()
        .to_string()
}

#[component]
pub fn Spinner() -> Element {
    rsx! {
        svg {
            class: "size-5 animate-spin",
            view_box: "0 0 24 24",
            fill: "none",
            circle {
                class: "opacity-25",
                cx: "12",
                cy: "12",
                r: "10",
                stroke: "currentColor",
                stroke_width: "4",
            }
            path {
                class: "opacity-75",
                fill: "currentColor",
                d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
            }
        }
    }
}

#[component]
pub fn Button(
    #[props(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[props(default = ButtonSize::MD)] size: ButtonSize,
    #[props(default = false)] loading: bool,
    #[props(default = false)] disabled: bool,
    #[props(default = "button".to_string())] button_type: String,
    #[props(default = String::new())] class: String,
    onclick: Option<EventHandler<MouseEvent>>,
    icon: Option<Element>,
    children: Element,
) -> Element {
    let base_classes = build_button_classes(variant, size, icon.is_some(), loading);
    let all_classes = if class.is_empty() {
        base_classes
    } else {
        format!("{} {}", base_classes, class)
    };
    let is_disabled = disabled || loading;

    rsx! {
        button {
            r#type: "{button_type}",
            class: "{all_classes}",
            disabled: is_disabled,
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
            },
            if loading {
                Spinner {}
            } else if let Some(icon_element) = icon {
                {icon_element}
            }
            {children}
        }
    }
}

#[component]
pub fn ButtonLink(
    to: crate::Route,
    #[props(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[props(default = ButtonSize::MD)] size: ButtonSize,
    #[props(default = String::new())] class: String,
    icon: Option<Element>,
    children: Element,
) -> Element {
    let base_classes = build_button_classes(variant, size, icon.is_some(), false);
    let all_classes = if class.is_empty() {
        base_classes
    } else {
        format!("{} {}", base_classes, class)
    };

    rsx! {
        Link { to, class: "{all_classes}",
            if let Some(icon_element) = icon {
                {icon_element}
            }
            {children}
        }
    }
}
