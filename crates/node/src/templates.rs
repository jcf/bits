use maud::{html, Markup, DOCTYPE};

// TODO: Enable when static-files is properly configured
// use static_files::Resource;
// include!(concat!(env!("OUT_DIR"), "/generated.rs"));

fn get_css_path() -> String {
    // For now, use a fixed path until we get the generated module working
    "/static/dist/app.css".to_string()
}

pub fn base(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                link rel="stylesheet" href=(get_css_path());
            }
            body class="bg-gray-50" {
                (content)
            }
        }
    }
}

pub fn navbar() -> Markup {
    html! {
        nav class="bg-white shadow-sm mb-8" {
            div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8" {
                div class="flex justify-between h-16" {
                    div class="flex space-x-8 items-center" {
                        a href="/" class="font-bold text-xl text-gray-900" { "🌐 Bits" }
                        div class="hidden md:flex space-x-4" {
                            a href="/" class="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium" { "Home" }
                            a href="/wallet" class="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium" { "Wallet" }
                            a href="/marketplace" class="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium" { "Marketplace" }
                            a href="/explorer" class="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium" { "Explorer" }
                        }
                    }
                }
            }
        }
    }
}

pub fn container(content: Markup) -> Markup {
    html! {
        div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8" {
            (navbar())
            (content)
        }
    }
}

pub fn card(title: &str, content: Markup) -> Markup {
    html! {
        div class="bg-white overflow-hidden shadow rounded-lg" {
            div class="px-4 py-5 sm:p-6" {
                @if !title.is_empty() {
                    h3 class="text-lg leading-6 font-medium text-gray-900 mb-4" { (title) }
                }
                (content)
            }
        }
    }
}

pub fn button(text: &str, onclick: &str) -> Markup {
    html! {
        button 
            type="button"
            onclick=(onclick)
            class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" {
            (text)
        }
    }
}

pub fn feature_card(icon: &str, title: &str, description: &str, link: &str, link_text: &str) -> Markup {
    html! {
        div class="relative bg-white p-6 rounded-lg shadow hover:shadow-lg transition-shadow" {
            div class="text-4xl mb-4" { (icon) }
            h3 class="text-lg font-medium text-gray-900 mb-2" { (title) }
            p class="text-gray-500 mb-4" { (description) }
            a href=(link) class="text-indigo-600 hover:text-indigo-500 font-medium" { (link_text) " →" }
        }
    }
}