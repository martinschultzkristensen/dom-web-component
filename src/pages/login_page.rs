use crate::services::supabase::{login, signup};
use crate::Route;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::use_navigator;

#[derive(Clone, PartialEq)]
enum AuthMode {
    Login,
    Signup,
}

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let email = use_state(|| String::new());
    let password = use_state(|| String::new());
    let confirm_password = use_state(|| String::new());
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let mode = use_state(|| AuthMode::Login);

    let navigator = use_navigator();

    let on_email_input = {
        let email = email.clone();

        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            email.set(input.value());
        })
    };

    let on_password_input = {
        let password = password.clone();

        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            password.set(input.value());
        })
    };

    let on_confirm_password_input = {
        let confirm_password = confirm_password.clone();

        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            confirm_password.set(input.value());
        })
    };

    let switch_to_login = {
        let mode = mode.clone();
        let error = error.clone();
        let success = success.clone();

        Callback::from(move |_| {
            error.set(None);
            success.set(None);
            mode.set(AuthMode::Login);
        })
    };

    let switch_to_signup = {
        let mode = mode.clone();
        let error = error.clone();
        let success = success.clone();

        Callback::from(move |_| {
            error.set(None);
            success.set(None);
            mode.set(AuthMode::Signup);
        })
    };

    let on_submit = {
        let email = email.clone();
        let password = password.clone();
        let confirm_password = confirm_password.clone();
        let error = error.clone();
        let success = success.clone();
        let loading = loading.clone();
        let mode = mode.clone();
        let navigator = navigator.clone();

        Callback::from(move |_| {
            let email_value = (*email).trim().to_string();
            let password_value = (*password).to_string();
            let confirm_password_value = (*confirm_password).to_string();
            let current_mode = (*mode).clone();

            if email_value.is_empty() || password_value.trim().is_empty() {
                error.set(Some("Please enter email and password.".to_string()));
                success.set(None);
                return;
            }

            if current_mode == AuthMode::Signup {
                if password_value.len() < 8 {
                    error.set(Some("Password must be at least 8 characters.".to_string()));
                    success.set(None);
                    return;
                }

                if password_value != confirm_password_value {
                    error.set(Some("Passwords do not match.".to_string()));
                    success.set(None);
                    return;
                }
            }

            error.set(None);
            success.set(None);
            loading.set(true);

            let error = error.clone();
            let success = success.clone();
            let loading = loading.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                match current_mode {
                    AuthMode::Login => {
                        match login(email_value, password_value).await {
                            Ok(_) => {
                                loading.set(false);

                                if let Some(nav) = navigator {
                                    nav.push(&Route::MainPage);
                                }
                            }
                            Err(message) => {
                                loading.set(false);
                                error.set(Some(message));
                            }
                        }
                    }
                    AuthMode::Signup => {
                        match signup(email_value, password_value).await {
                            Ok(Some(_session)) => {
                                loading.set(false);

                                if let Some(nav) = navigator {
                                    nav.push(&Route::MainPage);
                                }
                            }
                            Ok(None) => {
                                loading.set(false);
                                success.set(Some(
                                    "Account created. Please check your email to confirm your account before logging in."
                                        .to_string(),
                                ));
                            }
                            Err(message) => {
                                loading.set(false);
                                error.set(Some(message));
                            }
                        }
                    }
                }
            });
        })
    };

    let is_signup = *mode == AuthMode::Signup;

    html! {
        <div class="login-page">
            <h1>{ "DanceOmatic Creator" }</h1>

            <div class="login-box">
                <div class="auth-tabs">
                    <button
                        class={if !is_signup { "auth-tab active" } else { "auth-tab" }}
                        onclick={switch_to_login}
                    >
                        { "Login" }
                    </button>

                    <button
                        class={if is_signup { "auth-tab active" } else { "auth-tab" }}
                        onclick={switch_to_signup}
                    >
                        { "Create account" }
                    </button>
                </div>

                <h2>
                    {
                        if is_signup {
                            "Create account"
                        } else {
                            "Login"
                        }
                    }
                </h2>

                <p>
                    {
                        if is_signup {
                            "Create an account before creating dancers and choreographies. Your uploads will wait for admin approval before they can be used in DanceOmatic."
                        } else {
                            "Please log in before creating dancers and choreographies."
                        }
                    }
                </p>

                <input
                    type="email"
                    placeholder="Email"
                    value={(*email).clone()}
                    oninput={on_email_input}
                />

                <input
                    type="password"
                    placeholder="Password"
                    value={(*password).clone()}
                    oninput={on_password_input}
                />

                if is_signup {
                    <input
                        type="password"
                        placeholder="Confirm password"
                        value={(*confirm_password).clone()}
                        oninput={on_confirm_password_input}
                    />
                }

                if let Some(message) = &*error {
                    <p class="error-message">{ message }</p>
                }

                if let Some(message) = &*success {
                    <p class="success-message">{ message }</p>
                }

                <button class="login-submit-button" onclick={on_submit} disabled={*loading}>
                    {
                        if *loading {
                            if is_signup {
                                "Creating account..."
                            } else {
                                "Logging in..."
                            }
                        } else if is_signup {
                            "Create account"
                        } else {
                            "Login"
                        }
                    }
                </button>
            </div>
        </div>
    }
}