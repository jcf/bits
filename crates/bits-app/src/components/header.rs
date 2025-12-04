use dioxus::prelude::*;

use crate::auth::SessionState;
use crate::tenant::Realm;
use crate::Route;

use super::{Avatar, CategoryItem, MobileCategoryItem, NavigationPopover, SignOutLink};

#[component]
pub fn Header() -> Element {
    let realm = use_context::<Resource<Result<Realm>>>();
    let session = use_context::<Resource<Result<SessionState>>>();
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
        div { class: "bg-white dark:bg-black",
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
                        class: "fixed inset-0 bg-black/25 dark:bg-black/50",
                        onclick: move |_| mobile_menu_open.set(false),
                    }
                    // Dialog panel
                    div {
                        class: "fixed inset-0 z-40 flex",
                        tabindex: 0,
                        div {
                            class: "relative flex w-full max-w-xs transform flex-col overflow-y-auto bg-white pb-12 shadow-xl dark:bg-gray-900",
                            div { class: "flex px-4 pt-5 pb-2",
                                button {
                                    r#type: "button",
                                    class: "relative -m-2 inline-flex items-center justify-center rounded-md p-2 text-gray-400 hover:text-gray-500 dark:text-gray-500 dark:hover:text-gray-400",
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
                                div { class: "border-b border-gray-200 dark:border-gray-700",
                                    div { class: "-mb-px flex space-x-8 px-4",
                                        button {
                                            class: if mobile_tab_index() == 0 {
                                                "flex-1 border-b-2 border-indigo-600 px-1 py-4 text-base font-semibold whitespace-nowrap text-indigo-600 dark:border-indigo-400 dark:text-indigo-400"
                                            } else {
                                                "flex-1 border-b-2 border-transparent px-1 py-4 text-base font-semibold whitespace-nowrap text-gray-900 hover:text-gray-800 dark:text-gray-100 dark:hover:text-gray-200"
                                            },
                                            onclick: move |_| mobile_tab_index.set(0),
                                            "{t.t(\"nav-women\")}"
                                        }
                                        button {
                                            class: if mobile_tab_index() == 1 {
                                                "flex-1 border-b-2 border-indigo-600 px-1 py-4 text-base font-semibold whitespace-nowrap text-indigo-600 dark:border-indigo-400 dark:text-indigo-400"
                                            } else {
                                                "flex-1 border-b-2 border-transparent px-1 py-4 text-base font-semibold whitespace-nowrap text-gray-900 hover:text-gray-800 dark:text-gray-100 dark:hover:text-gray-200"
                                            },
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

                            div { class: "space-y-6 border-t border-gray-200 px-4 py-6 dark:border-gray-700",
                                div { class: "flow-root",
                                    span { class: "-m-2 block p-2 font-medium text-gray-900 dark:text-white", "{t.t(\"nav-company\")}" }
                                }
                                div { class: "flow-root",
                                    span { class: "-m-2 block p-2 font-medium text-gray-900 dark:text-white", "{t.t(\"nav-stores\")}" }
                                }
                            }

                            div { class: "space-y-6 border-t border-gray-200 px-4 py-6 dark:border-gray-700",
                                match session() {
                                    Some(Ok(SessionState::Authenticated(user))) => rsx! {
                                        div { class: "flow-root",
                                            div { class: "-m-2 flex items-center p-2",
                                                Avatar { email: user.email.clone() }
                                                span { class: "ml-3 text-sm font-medium text-gray-900 dark:text-white", "{user.email}" }
                                            }
                                        }
                                        div { class: "flow-root",
                                            SignOutLink { class: "-m-2 block p-2 font-medium text-gray-900 hover:text-gray-800 dark:text-white dark:hover:text-gray-200" }
                                        }
                                    },
                                    _ => rsx! {
                                        div { class: "flow-root",
                                            Link {
                                                to: Route::Join {},
                                                class: "-m-2 block p-2 font-medium text-gray-900 hover:text-gray-800 dark:text-white dark:hover:text-gray-200",
                                                "{t.t(\"auth-create-account-link\")}"
                                            }
                                        }
                                        div { class: "flow-root",
                                            Link {
                                                to: Route::Auth {},
                                                class: "-m-2 block p-2 font-medium text-gray-900 hover:text-gray-800 dark:text-white dark:hover:text-gray-200",
                                                "{t.t(\"auth-sign-in-button\")}"
                                            }
                                        }
                                    },
                                }
                            }

                            div { class: "space-y-6 border-t border-gray-200 px-4 py-6 dark:border-gray-700",
                                form {
                                    div { class: "-ml-2 inline-grid grid-cols-1",
                                        select {
                                            id: "mobile-currency",
                                            name: "currency",
                                            "aria-label": "Currency",
                                            class: "col-start-1 row-start-1 w-full appearance-none rounded-md bg-white py-0.5 pr-7 pl-2 text-base font-medium text-gray-700 focus:outline-2 dark:bg-gray-800 dark:text-gray-300 dark:focus:outline-indigo-500",
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
                                            class: "pointer-events-none col-start-1 row-start-1 mr-1 size-5 self-center justify-self-end fill-gray-500 dark:fill-gray-400",
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
                    div { class: "bg-gray-900 dark:bg-gray-950",
                        div { class: "mx-auto flex h-10 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8",
                            form {
                                div { class: "-ml-2 inline-grid grid-cols-1",
                                    select {
                                        id: "desktop-currency",
                                        name: "currency",
                                        "aria-label": "Currency",
                                        class: "col-start-1 row-start-1 w-full appearance-none rounded-md bg-gray-900 py-0.5 pr-7 pl-2 text-left text-base font-medium text-white focus:outline-2 focus:-outline-offset-1 focus:outline-white dark:bg-gray-950 dark:focus:outline-indigo-400",
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
                                        class: "pointer-events-none col-start-1 row-start-1 mr-1 size-5 self-center justify-self-end fill-gray-300 dark:fill-gray-400",
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
                                    Some(Ok(SessionState::Authenticated(user))) => rsx! {
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
                    div { class: "bg-white dark:bg-gray-800",
                        div { class: "mx-auto max-w-7xl px-4 sm:px-6 lg:px-8",
                            div { class: "border-b border-gray-200 dark:border-gray-700",
                                div { class: "flex h-16 items-center justify-between",
                                    // Logo (lg+)
                                    div { class: "hidden lg:flex lg:flex-1 lg:items-center",
                                        Link {
                                            to: Route::Home {},
                                            span { class: "sr-only", "{t.t(\"brand-company-name\")}" }
                                            div { class: "size-8 rounded bg-gray-100 flex items-center justify-center text-gray-500 font-bold dark:bg-gray-900 dark:text-gray-400", "B"}
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

                                                a { class: "flex items-center text-sm font-semibold text-gray-700 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white", "{t.t(\"nav-company\")}" }
                                                a { class: "flex items-center text-sm font-semibold text-gray-700 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white", "{t.t(\"nav-stores\")}" }
                                            }
                                        }
                                    }

                                    // Mobile menu and search (lg-)
                                    div { class: "flex flex-1 items-center lg:hidden",
                                        button {
                                            r#type: "button",
                                            class: "-ml-2 rounded-md bg-white p-2 text-gray-400 hover:text-gray-500 dark:bg-gray-800 dark:text-gray-500 dark:hover:text-gray-400",
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
                                            class: "ml-2 p-2 text-gray-400 hover:text-gray-500 dark:text-gray-500 dark:hover:text-gray-400",
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
                                        div { class: "size-8 bg-gray-100 rounded flex items-center justify-center text-gray-500 font-bold dark:bg-gray-900 dark:text-gray-400", "B" }
                                    }

                                    div { class: "flex flex-1 items-center justify-end",
                                        button {
                                            r#type: "button",
                                            class: "hidden text-sm font-semibold text-gray-700 hover:text-gray-900 lg:block dark:text-gray-300 dark:hover:text-white",
                                            "{t.t(\"common-search\")}"
                                        }

                                        div { class: "flex items-center lg:ml-8",
                                            // Help
                                            button {
                                                r#type: "button",
                                                class: "p-2 text-gray-400 hover:text-gray-500 lg:hidden dark:text-gray-500 dark:hover:text-gray-400",
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
                                                class: "hidden text-sm font-semibold text-gray-700 hover:text-gray-900 lg:block dark:text-gray-300 dark:hover:text-white",
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
                                                        class: "size-6 shrink-0 text-gray-400 group-hover:text-gray-500 dark:text-gray-500 dark:group-hover:text-gray-400",
                                                        path {
                                                            d: "M15.75 10.5V6a3.75 3.75 0 1 0-7.5 0v4.5m11.356-1.993 1.263 12c.07.665-.45 1.243-1.119 1.243H4.25a1.125 1.125 0 0 1-1.12-1.243l1.264-12A1.125 1.125 0 0 1 5.513 7.5h12.974c.576 0 1.059.435 1.119 1.007ZM8.625 10.5a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Zm7.5 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Z",
                                                            stroke_linecap: "round",
                                                            stroke_linejoin: "round",
                                                        }
                                                    }
                                                    span { class: "ml-2 text-sm font-medium text-gray-700 group-hover:text-gray-900 dark:text-gray-300 dark:group-hover:text-white", "0" }
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
