use leptos::prelude::*;

#[component]
pub fn Avatar(
    #[prop(optional, into)] name: String,
    #[prop(optional, into)] url: Option<String>,
    #[prop(optional, into)] class: String,
    #[prop(default = "md".to_string())] size: String,
) -> impl IntoView {
    let size_class = match size.as_str() {
        "sm" => "avatar-sm",
        "lg" => "avatar-lg",
        "xl" => "avatar-xl",
        _ => "",
    };
    let initial = name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();

    view! {
        <div class={format!("avatar {size_class} {class}")}>
            {match url {
                Some(u) if !u.is_empty() => view! { <img src={u} alt={name.clone()} /> }.into_any(),
                _ => view! { <span>{initial}</span> }.into_any(),
            }}
        </div>
    }
}
