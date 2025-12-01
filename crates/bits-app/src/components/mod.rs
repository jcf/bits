pub mod button;

use dioxus::prelude::*;

pub use button::{Button, ButtonLink, ButtonSize, ButtonVariant, Spinner};

use crate::auth::User;
use crate::tenant::Realm;
use crate::Route;

#[component]
pub fn Input(id: String, input_type: String, name: String, placeholder: String) -> Element {
    let autocapitalize = if input_type == "email" {
        Some("none")
    } else {
        None
    };

    rsx! {
        input {
            id: "{id}",
            r#type: "{input_type}",
            name: "{name}",
            required: true,
            autocomplete: "off",
            autocapitalize,
            placeholder: "{placeholder}",
            class: "block w-full rounded-md px-3 py-2 border border-neutral-300 dark:border-neutral-700 text-neutral-900 dark:text-neutral-100",
        }
    }
}

#[component]
pub fn SignOutLink(class: Option<String>) -> Element {
    use crate::auth::sign_out;
    let mut session = use_context::<Resource<Result<Option<User>>>>();
    let mut sign_out_action = use_action(sign_out);
    let nav = navigator();
    let t = crate::i18n::use_translation();

    use_effect(move || {
        if sign_out_action.value().and_then(|r| r.ok()).is_some() {
            session.restart();
            nav.push(Route::Home {});
        }
    });
    rsx! {
        button {
            r#type: "button",
            class: class.unwrap_or_default(),
            onclick: move |_| sign_out_action.call(),
            "{t.t(\"auth-sign-out-button\")}"
        }
    }
}

#[component]
fn Avatar(email: String) -> Element {
    rsx! {
        span {
            class: "inline-block size-6 overflow-hidden rounded-full bg-gray-100 outline -outline-offset-1 outline-black/5 dark:bg-gray-800 dark:outline-white/10",
            svg {
                view_box: "0 0 24 24",
                fill: "currentColor",
                class: "size-full text-gray-300 dark:text-gray-600",
                path {
                    d: "M24 20.993V24H0v-2.996A14.977 14.977 0 0112.004 15c4.904 0 9.26 2.354 11.996 5.993zM16.002 8.999a4 4 0 11-8 0 4 4 0 018 0z",
                }
            }
        }
    }
}

#[component]
pub fn Header() -> Element {
    let realm = use_context::<Resource<Result<Realm>>>();
    let session = use_context::<Resource<Result<Option<User>>>>();
    let mut mobile_menu_open = use_signal(|| false);
    let mut mobile_tab_index = use_signal(|| 0);
    let mut women_popover_open = use_signal(|| false);
    let mut men_popover_open = use_signal(|| false);
    let mut switching_popover = use_signal(|| false);
    let t = crate::i18n::use_translation();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        use_effect(move || {
            if !women_popover_open() && !men_popover_open() {
                return;
            }

            let document = web_sys::window()
                .and_then(|w| w.document())
                .expect("no document");

            let handler = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                if let Some(target) = event.target() {
                    if let Some(element) = target.dyn_ref::<web_sys::Element>() {
                        let mut is_inside_popover = false;
                        let mut node = Some(element.clone());

                        while let Some(current) = node {
                            if current.has_attribute("data-popover") {
                                is_inside_popover = true;
                                break;
                            }
                            node = current.parent_element();
                        }

                        if !is_inside_popover {
                            women_popover_open.set(false);
                            men_popover_open.set(false);
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);

            let _ = document.add_event_listener_with_callback_and_bool(
                "click",
                handler.as_ref().unchecked_ref(),
                true,
            );
            handler.forget();
        });
    }

    rsx! {
        div { class: "bg-white",
            {
                match realm() {
                    Some(Ok(Realm::Tenancy(tenant))) => rsx! {
                        document::Title { "{tenant.name}" }
                    },
                    _ => rsx! {},
                }
            }

            // Mobile menu
            if mobile_menu_open() {
                div {
                    class: "relative z-40 lg:hidden",
                    role: "dialog",
                    "aria-modal": "true",
                    // Backdrop
                    div {
                        class: "fixed inset-0 bg-black/25",
                        onclick: move |_| mobile_menu_open.set(false),
                    }
                    // Dialog panel
                    div {
                        class: "fixed inset-0 z-40 flex",
                        tabindex: 0,
                        div {
                            class: "relative flex w-full max-w-xs transform flex-col overflow-y-auto bg-white pb-12 shadow-xl",
                            div { class: "flex px-4 pt-5 pb-2",
                                button {
                                    r#type: "button",
                                    class: "relative -m-2 inline-flex items-center justify-center rounded-md p-2 text-gray-400",
                                    onclick: move |_| mobile_menu_open.set(false),
                                    span { class: "absolute -inset-0.5" }
                                    span { class: "sr-only", "{t.t(\"common-close\")}" }
                                    svg {
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "1.5",
                                        "aria-hidden": "true",
                                        class: "size-6",
                                        path {
                                            d: "M6 18 18 6M6 6l12 12",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                        }
                                    }
                                }
                            }

                            // Mobile tabs
                            div { class: "mt-2",
                                div { class: "border-b border-gray-200",
                                    div { class: "-mb-px flex space-x-8 px-4",
                                        button {
                                            class: if mobile_tab_index() == 0 { "flex-1 border-b-2 border-indigo-600 px-1 py-4 text-base font-medium whitespace-nowrap text-indigo-600" } else { "flex-1 border-b-2 border-transparent px-1 py-4 text-base font-medium whitespace-nowrap text-gray-900" },
                                            onclick: move |_| mobile_tab_index.set(0),
                                            "{t.t(\"nav-women\")}"
                                        }
                                        button {
                                            class: if mobile_tab_index() == 1 { "flex-1 border-b-2 border-indigo-600 px-1 py-4 text-base font-medium whitespace-nowrap text-indigo-600" } else { "flex-1 border-b-2 border-transparent px-1 py-4 text-base font-medium whitespace-nowrap text-gray-900" },
                                            onclick: move |_| mobile_tab_index.set(1),
                                            "{t.t(\"nav-men\")}"
                                        }
                                    }
                                }
                                // Tab panels
                                if mobile_tab_index() == 0 {
                                    div { class: "space-y-12 px-4 py-6",
                                        div { class: "grid grid-cols-2 gap-x-4 gap-y-10",
                                            MobileCategoryItem { name: t.t("category-new-arrivals") }
                                            MobileCategoryItem { name: t.t("category-basic-tees") }
                                            MobileCategoryItem { name: t.t("category-accessories") }
                                            MobileCategoryItem { name: t.t("category-carry") }
                                        }
                                    }
                                }
                                if mobile_tab_index() == 1 {
                                    div { class: "space-y-12 px-4 py-6",
                                        div { class: "grid grid-cols-2 gap-x-4 gap-y-10",
                                            MobileCategoryItem { name: t.t("category-new-arrivals") }
                                            MobileCategoryItem { name: t.t("category-basic-tees") }
                                            MobileCategoryItem { name: t.t("category-accessories") }
                                            MobileCategoryItem { name: t.t("category-carry") }
                                        }
                                    }
                                }
                            }

                            div { class: "space-y-6 border-t border-gray-200 px-4 py-6",
                                div { class: "flow-root",
                                    span { class: "-m-2 block p-2 font-medium text-gray-900", "{t.t(\"nav-company\")}" }
                                }
                                div { class: "flow-root",
                                    span { class: "-m-2 block p-2 font-medium text-gray-900", "{t.t(\"nav-stores\")}" }
                                }
                            }

                            div { class: "space-y-6 border-t border-gray-200 px-4 py-6",
                                match session() {
                                    Some(Ok(Some(user))) => rsx! {
                                        div { class: "flow-root",
                                            div { class: "-m-2 flex items-center p-2",
                                                Avatar { email: user.email.clone() }
                                                span { class: "ml-3 text-sm font-medium text-gray-900", "{user.email}" }
                                            }
                                        }
                                        div { class: "flow-root",
                                            SignOutLink { class: "-m-2 block p-2 font-medium text-gray-900" }
                                        }
                                    },
                                    _ => rsx! {
                                        div { class: "flow-root",
                                            Link {
                                                to: Route::Join {},
                                                class: "-m-2 block p-2 font-medium text-gray-900",
                                                "{t.t(\"auth-create-account-link\")}"
                                            }
                                        }
                                        div { class: "flow-root",
                                            Link {
                                                to: Route::Auth {},
                                                class: "-m-2 block p-2 font-medium text-gray-900",
                                                "{t.t(\"auth-sign-in-button\")}"
                                            }
                                        }
                                    },
                                }
                            }

                            div { class: "space-y-6 border-t border-gray-200 px-4 py-6",
                                form {
                                    div { class: "-ml-2 inline-grid grid-cols-1",
                                        select {
                                            id: "mobile-currency",
                                            name: "currency",
                                            "aria-label": "Currency",
                                            class: "col-start-1 row-start-1 w-full appearance-none rounded-md bg-white py-0.5 pr-7 pl-2 text-base font-medium text-gray-700 group-hover:text-gray-800 focus:outline-2 sm:text-sm/6",
                                            option { "CAD" }
                                            option { "USD" }
                                            option { "AUD" }
                                            option { "EUR" }
                                            option { "GBP" }
                                        }
                                        svg {
                                            view_box: "0 0 20 20",
                                            fill: "currentColor",
                                            "aria-hidden": "true",
                                            class: "pointer-events-none col-start-1 row-start-1 mr-1 size-5 self-center justify-self-end fill-gray-500",
                                            path {
                                                d: "M5.22 8.22a.75.75 0 0 1 1.06 0L10 11.94l3.72-3.72a.75.75 0 1 1 1.06 1.06l-4.25 4.25a.75.75 0 0 1-1.06 0L5.22 9.28a.75.75 0 0 1 0-1.06Z",
                                                clip_rule: "evenodd",
                                                fill_rule: "evenodd",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            header { class: "relative",
                nav { "aria-label": "Top",
                    // Top navigation
                    div { class: "bg-gray-900",
                        div { class: "mx-auto flex h-10 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8",
                            form {
                                div { class: "-ml-2 inline-grid grid-cols-1",
                                    select {
                                        id: "desktop-currency",
                                        name: "currency",
                                        "aria-label": "Currency",
                                        class: "col-start-1 row-start-1 w-full appearance-none rounded-md bg-gray-900 py-0.5 pr-7 pl-2 text-left text-base font-medium text-white focus:outline-2 focus:-outline-offset-1 focus:outline-white sm:text-sm/6",
                                        option { "CAD" }
                                        option { "USD" }
                                        option { "AUD" }
                                        option { "EUR" }
                                        option { "GBP" }
                                    }
                                    svg {
                                        view_box: "0 0 20 20",
                                        fill: "currentColor",
                                        "aria-hidden": "true",
                                        class: "pointer-events-none col-start-1 row-start-1 mr-1 size-5 self-center justify-self-end fill-gray-300",
                                        path {
                                            d: "M5.22 8.22a.75.75 0 0 1 1.06 0L10 11.94l3.72-3.72a.75.75 0 1 1 1.06 1.06l-4.25 4.25a.75.75 0 0 1-1.06 0L5.22 9.28a.75.75 0 0 1 0-1.06Z",
                                            clip_rule: "evenodd",
                                            fill_rule: "evenodd",
                                        }
                                    }
                                }
                            }

                            div { class: "flex items-center space-x-6",
                                match session() {
                                    Some(Ok(Some(user))) => rsx! {
                                        div { class: "flex items-center",
                                            Avatar { email: user.email.clone() }
                                            span { class: "ml-3 text-sm font-medium text-white", "{user.email}" }
                                        }
                                        SignOutLink { class: "text-sm font-medium text-white hover:text-gray-100" }
                                    },
                                    _ => rsx! {
                                        Link {
                                            to: Route::Auth {},
                                            class: "text-sm font-medium text-white hover:text-gray-100",
                                            "{t.t(\"auth-sign-in-button\")}"
                                        }
                                        Link {
                                            to: Route::Join {},
                                            class: "text-sm font-medium text-white hover:text-gray-100",
                                            "{t.t(\"auth-create-account-link\")}"
                                        }
                                    },
                                }
                            }
                        }
                    }

                    // Secondary navigation
                    div { class: "bg-white",
                        div { class: "mx-auto max-w-7xl px-4 sm:px-6 lg:px-8",
                            div { class: "border-b border-gray-200",
                                div { class: "flex h-16 items-center justify-between",
                                    // Logo (lg+)
                                    div { class: "hidden lg:flex lg:flex-1 lg:items-center",
                                        Link {
                                            to: Route::Home {},
                                            span { class: "sr-only", "{t.t(\"brand-company-name\")}" }
                                            img {
                                                src: "https://tailwindcss.com/plus-assets/img/logos/mark.svg?color=indigo&shade=600",
                                                alt: "",
                                                class: "h-8 w-auto",
                                            }
                                        }
                                    }

                                    div { class: "hidden h-full lg:flex",
                                        div { class: "inset-x-0 bottom-0 px-4",
                                            div { class: "flex h-full justify-center space-x-8",
                                                NavigationPopover {
                                                    label: t.t("nav-women"),
                                                    is_open: women_popover_open,
                                                    switching: switching_popover,
                                                    on_toggle: move |_| {
                                                        if men_popover_open() && !women_popover_open() {
                                                            switching_popover.set(true);
                                                        }
                                                        men_popover_open.set(false);
                                                        women_popover_open.set(!women_popover_open());
                                                    },
                                                    div { class: "grid grid-cols-4 gap-x-8 gap-y-10 py-16",
                                                        CategoryItem { name: "Dresses & Skirts".to_string() }
                                                        CategoryItem { name: "Tops & Blouses".to_string() }
                                                        CategoryItem { name: "Handbags".to_string() }
                                                        CategoryItem { name: "Jewelry".to_string() }
                                                    }
                                                }

                                                NavigationPopover {
                                                    label: t.t("nav-men"),
                                                    is_open: men_popover_open,
                                                    switching: switching_popover,
                                                    on_toggle: move |_| {
                                                        if women_popover_open() && !men_popover_open() {
                                                            switching_popover.set(true);
                                                        }
                                                        women_popover_open.set(false);
                                                        men_popover_open.set(!men_popover_open());
                                                    },
                                                    div { class: "grid grid-cols-4 gap-x-8 gap-y-10 py-16",
                                                        CategoryItem { name: "Shirts & Tops".to_string() }
                                                        CategoryItem { name: "Pants & Jeans".to_string() }
                                                        CategoryItem { name: "Outerwear".to_string() }
                                                        CategoryItem { name: "Shoes".to_string() }
                                                    }
                                                }

                                                a { class: "flex items-center text-sm font-medium text-gray-700 hover:text-gray-800", "{t.t(\"nav-company\")}" }
                                                a { class: "flex items-center text-sm font-medium text-gray-700 hover:text-gray-800", "{t.t(\"nav-stores\")}" }
                                            }
                                        }
                                    }

                                    // Mobile menu and search (lg-)
                                    div { class: "flex flex-1 items-center lg:hidden",
                                        button {
                                            r#type: "button",
                                            class: "-ml-2 rounded-md bg-white p-2 text-gray-400",
                                            onclick: move |_| mobile_menu_open.set(true),
                                            span { class: "sr-only", "{t.t(\"mobile-menu-open\")}" }
                                            svg {
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "1.5",
                                                "aria-hidden": "true",
                                                class: "size-6",
                                                path {
                                                    d: "M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                }
                                            }
                                        }

                                        button {
                                            r#type: "button",
                                            class: "ml-2 p-2 text-gray-400 hover:text-gray-500",
                                            span { class: "sr-only", "{t.t(\"common-search\")}" }
                                            svg {
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "1.5",
                                                "aria-hidden": "true",
                                                class: "size-6",
                                                path {
                                                    d: "m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                }
                                            }
                                        }
                                    }

                                    // Logo (lg-)
                                    Link {
                                        to: Route::Home {},
                                        class: "lg:hidden",
                                        span { class: "sr-only", "{t.t(\"brand-company-name\")}" }
                                        img {
                                            src: "https://tailwindcss.com/plus-assets/img/logos/mark.svg?color=indigo&shade=600",
                                            alt: "",
                                            class: "h-8 w-auto",
                                        }
                                    }

                                    div { class: "flex flex-1 items-center justify-end",
                                        button {
                                            r#type: "button",
                                            class: "hidden text-sm font-medium text-gray-700 hover:text-gray-800 lg:block",
                                            "{t.t(\"common-search\")}"
                                        }

                                        div { class: "flex items-center lg:ml-8",
                                            // Help
                                            button {
                                                r#type: "button",
                                                class: "p-2 text-gray-400 hover:text-gray-500 lg:hidden",
                                                span { class: "sr-only", "{t.t(\"common-help\")}" }
                                                svg {
                                                    view_box: "0 0 24 24",
                                                    fill: "none",
                                                    stroke: "currentColor",
                                                    stroke_width: "1.5",
                                                    "aria-hidden": "true",
                                                    class: "size-6",
                                                    path {
                                                        d: "M9.879 7.519c1.171-1.025 3.071-1.025 4.242 0 1.172 1.025 1.172 2.687 0 3.712-.203.179-.43.326-.67.442-.745.361-1.45.999-1.45 1.827v.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 5.25h.008v.008H12v-.008Z",
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                    }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "hidden text-sm font-medium text-gray-700 hover:text-gray-800 lg:block",
                                                "{t.t(\"common-help\")}"
                                            }

                                            // Cart
                                            div { class: "ml-4 flow-root lg:ml-8",
                                                button {
                                                    r#type: "button",
                                                    class: "group -m-2 flex items-center p-2",
                                                    svg {
                                                        view_box: "0 0 24 24",
                                                        fill: "none",
                                                        stroke: "currentColor",
                                                        stroke_width: "1.5",
                                                        "aria-hidden": "true",
                                                        class: "size-6 shrink-0 text-gray-400 group-hover:text-gray-500",
                                                        path {
                                                            d: "M15.75 10.5V6a3.75 3.75 0 1 0-7.5 0v4.5m11.356-1.993 1.263 12c.07.665-.45 1.243-1.119 1.243H4.25a1.125 1.125 0 0 1-1.12-1.243l1.264-12A1.125 1.125 0 0 1 5.513 7.5h12.974c.576 0 1.059.435 1.119 1.007ZM8.625 10.5a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Zm7.5 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Z",
                                                            stroke_linecap: "round",
                                                            stroke_linejoin: "round",
                                                        }
                                                    }
                                                    span { class: "ml-2 text-sm font-medium text-gray-700 group-hover:text-gray-800", "0" }
                                                    span { class: "sr-only", "{t.t(\"common-cart-items\")}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NavigationPopover(
    label: String,
    is_open: Signal<bool>,
    switching: Signal<bool>,
    on_toggle: EventHandler<()>,
    children: Element,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        use_effect(move || {
            if switching() {
                let window = web_sys::window().expect("no window");
                let mut switching = switching;
                let closure = Closure::wrap(Box::new(move || {
                    switching.set(false);
                }) as Box<dyn FnMut()>);
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    50,
                );
                closure.forget();
            }
        });
    }

    rsx! {
        div { class: "flex", "data-popover": "{label}",
            div { class: "relative flex",
                button {
                    r#type: "button",
                    class: if is_open() {
                        "group relative flex items-center justify-center text-sm font-medium transition-colors duration-200 ease-out text-indigo-600"
                    } else {
                        "group relative flex items-center justify-center text-sm font-medium text-gray-700 transition-colors duration-200 ease-out hover:text-gray-800"
                    },
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_toggle.call(());
                    },
                    "{label}"
                    span {
                        "aria-hidden": "true",
                        class: if is_open() {
                            "absolute inset-x-0 -bottom-px z-30 h-0.5 transition duration-200 ease-out bg-indigo-600"
                        } else {
                            "absolute inset-x-0 -bottom-px z-30 h-0.5 transition duration-200 ease-out"
                        },
                    }
                }
            }
            div {
                class: if switching() {
                    if is_open() {
                        "absolute inset-x-0 top-full z-20 w-full bg-white text-sm text-gray-500 opacity-100 translate-y-0"
                    } else {
                        "absolute inset-x-0 top-full z-20 w-full bg-white text-sm text-gray-500 opacity-0 translate-y-0 pointer-events-none"
                    }
                } else {
                    if is_open() {
                        "absolute inset-x-0 top-full z-20 w-full bg-white text-sm text-gray-500 transition-all duration-200 ease-out opacity-100 translate-y-0"
                    } else {
                        "absolute inset-x-0 top-full z-20 w-full bg-white text-sm text-gray-500 transition-all duration-150 ease-in opacity-0 translate-y-1 pointer-events-none"
                    }
                },
                div { "aria-hidden": "true", class: "absolute inset-0 top-1/2 bg-white shadow-sm" }
                div { class: "relative bg-white",
                    div { class: "mx-auto max-w-7xl px-8",
                        {children}
                    }
                }
            }
        }
    }
}

#[component]
fn CategoryItem(name: String) -> Element {
    let t = crate::i18n::use_translation();

    rsx! {
        div { class: "group relative",
            div { class: "aspect-square w-full rounded-md bg-gray-200" }
            div { class: "mt-4 block font-medium text-gray-900",
                span { "aria-hidden": "true", class: "absolute inset-0 z-10" }
                "{name}"
            }
            p { "aria-hidden": "true", class: "mt-1", "{t.t(\"category-shop-now\")}" }
        }
    }
}

#[component]
fn MobileCategoryItem(name: String) -> Element {
    let t = crate::i18n::use_translation();

    rsx! {
        div { class: "group relative",
            div { class: "aspect-square w-full rounded-md bg-gray-200" }
            div { class: "mt-6 block text-sm font-medium text-gray-900",
                span { "aria-hidden": "true", class: "absolute inset-0 z-10" }
                "{name}"
            }
            p { "aria-hidden": "true", class: "mt-1 text-sm text-gray-500", "{t.t(\"category-shop-now\")}" }
        }
    }
}

#[component]
pub fn Footer() -> Element {
    let realm = use_context::<Resource<Result<Realm>>>();
    let t = crate::i18n::use_translation();

    rsx! {
        footer { class: "bg-neutral-100 dark:bg-neutral-900",
            div { class: "mx-auto max-w-7xl px-6 py-12 md:flex md:items-center md:justify-between lg:px-8",
                div { class: "flex justify-center gap-x-6 md:order-2",
                    button {
                        r#type: "button",
                        class: "text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-white",
                        span { class: "sr-only", "{t.t(\"social-facebook\")}" }
                        svg {
                            view_box: "0 0 24 24",
                            fill: "currentColor",
                            "aria-hidden": "true",
                            class: "size-6",
                            path {
                                d: "M22 12c0-5.523-4.477-10-10-10S2 6.477 2 12c0 4.991 3.657 9.128 8.438 9.878v-6.987h-2.54V12h2.54V9.797c0-2.506 1.492-3.89 3.777-3.89 1.094 0 2.238.195 2.238.195v2.46h-1.26c-1.243 0-1.63.771-1.63 1.562V12h2.773l-.443 2.89h-2.33v6.988C18.343 21.128 22 16.991 22 12z",
                                "clip-rule": "evenodd",
                                "fill-rule": "evenodd",
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-white",
                        span { class: "sr-only", "{t.t(\"social-instagram\")}" }
                        svg {
                            view_box: "0 0 24 24",
                            fill: "currentColor",
                            "aria-hidden": "true",
                            class: "size-6",
                            path {
                                d: "M12.315 2c2.43 0 2.784.013 3.808.06 1.064.049 1.791.218 2.427.465a4.902 4.902 0 011.772 1.153 4.902 4.902 0 011.153 1.772c.247.636.416 1.363.465 2.427.048 1.067.06 1.407.06 4.123v.08c0 2.643-.012 2.987-.06 4.043-.049 1.064-.218 1.791-.465 2.427a4.902 4.902 0 01-1.153 1.772 4.902 4.902 0 01-1.772 1.153c-.636.247-1.363.416-2.427.465-1.067.048-1.407.06-4.123.06h-.08c-2.643 0-2.987-.012-4.043-.06-1.064-.049-1.791-.218-2.427-.465a4.902 4.902 0 01-1.772-1.153 4.902 4.902 0 01-1.153-1.772c-.247-.636-.416-1.363-.465-2.427-.047-1.024-.06-1.379-.06-3.808v-.63c0-2.43.013-2.784.06-3.808.049-1.064.218-1.791.465-2.427a4.902 4.902 0 011.153-1.772A4.902 4.902 0 015.45 2.525c.636-.247 1.363-.416 2.427-.465C8.901 2.013 9.256 2 11.685 2h.63zm-.081 1.802h-.468c-2.456 0-2.784.011-3.807.058-.975.045-1.504.207-1.857.344-.467.182-.8.398-1.15.748-.35.35-.566.683-.748 1.15-.137.353-.3.882-.344 1.857-.047 1.023-.058 1.351-.058 3.807v.468c0 2.456.011 2.784.058 3.807.045.975.207 1.504.344 1.857.182.466.399.8.748 1.15.35.35.683.566 1.15.748.353.137.882.3 1.857.344 1.054.048 1.37.058 4.041.058h.08c2.597 0 2.917-.01 3.96-.058.976-.045 1.505-.207 1.858-.344.466-.182.8-.398 1.15-.748.35-.35.566-.683.748-1.15.137-.353.3-.882.344-1.857.048-1.055.058-1.37.058-4.041v-.08c0-2.597-.01-2.917-.058-3.96-.045-.976-.207-1.505-.344-1.858a3.097 3.097 0 00-.748-1.15 3.098 3.098 0 00-1.15-.748c-.353-.137-.882-.3-1.857-.344-1.023-.047-1.351-.058-3.807-.058zM12 6.865a5.135 5.135 0 110 10.27 5.135 5.135 0 010-10.27zm0 1.802a3.333 3.333 0 100 6.666 3.333 3.333 0 000-6.666zm5.338-3.205a1.2 1.2 0 110 2.4 1.2 1.2 0 010-2.4z",
                                "clip-rule": "evenodd",
                                "fill-rule": "evenodd",
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-white",
                        span { class: "sr-only", "{t.t(\"social-x\")}" }
                        svg {
                            view_box: "0 0 24 24",
                            fill: "currentColor",
                            "aria-hidden": "true",
                            class: "size-6",
                            path {
                                d: "M13.6823 10.6218L20.2391 3H18.6854L12.9921 9.61788L8.44486 3H3.2002L10.0765 13.0074L3.2002 21H4.75404L10.7663 14.0113L15.5685 21H20.8131L13.6819 10.6218H13.6823ZM11.5541 13.0956L10.8574 12.0991L5.31391 4.16971H7.70053L12.1742 10.5689L12.8709 11.5655L18.6861 19.8835H16.2995L11.5541 13.096V13.0956Z",
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-white",
                        span { class: "sr-only", "{t.t(\"social-github\")}" }
                        svg {
                            view_box: "0 0 24 24",
                            fill: "currentColor",
                            "aria-hidden": "true",
                            class: "size-6",
                            path {
                                d: "M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z",
                                "clip-rule": "evenodd",
                                "fill-rule": "evenodd",
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-white",
                        span { class: "sr-only", "{t.t(\"social-youtube\")}" }
                        svg {
                            view_box: "0 0 24 24",
                            fill: "currentColor",
                            "aria-hidden": "true",
                            class: "size-6",
                            path {
                                d: "M19.812 5.418c.861.23 1.538.907 1.768 1.768C21.998 8.746 22 12 22 12s0 3.255-.418 4.814a2.504 2.504 0 0 1-1.768 1.768c-1.56.419-7.814.419-7.814.419s-6.255 0-7.814-.419a2.505 2.505 0 0 1-1.768-1.768C2 15.255 2 12 2 12s0-3.255.417-4.814a2.507 2.507 0 0 1 1.768-1.768C5.744 5 11.998 5 11.998 5s6.255 0 7.814.418ZM15.194 12 10 15V9l5.194 3Z",
                                "clip-rule": "evenodd",
                                "fill-rule": "evenodd",
                            }
                        }
                    }
                }
                p { class: "mt-8 text-center text-sm/6 text-gray-600 md:order-1 md:mt-0 dark:text-gray-400",
                    match realm() {
                        Some(Ok(Realm::Tenancy(tenant))) => rsx! {
                            "{tenant.name}"
                        },
                        Some(Ok(_)) => rsx! {
                            "{t.t(\"brand-name\")}"
                        },
                        Some(Err(_)) => rsx! {
                            "{t.t(\"brand-name\")}"
                        },
                        None => rsx! {
                            "{t.t(\"common-loading\")}"
                        },
                    }
                }
            }
        }
    }
}
