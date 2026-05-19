use web_sys::HtmlInputElement;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;
use crate::User;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(|| String::new());
    let user = use_context::<User>().expect("No context found.");

    let oninput = {
        let current_username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_username.set(input.value());
        })
    };

    let onclick = {
        let username = username.clone();
        let user = user.clone();
        Callback::from(move |_| *user.username.borrow_mut() = (*username).trim().to_string())
    };

    html! {
        <div class="min-h-screen w-screen bg-gray-100 flex items-center justify-center px-6">
            <div class="w-full max-w-sm bg-white rounded-lg shadow-md border border-gray-200 p-6">
                <div class="text-2xl font-bold text-gray-900 text-center">{"YewChat"}</div>
                <div class="text-sm text-gray-500 text-center mt-2">{"Enter a nickname to join the chat."}</div>

                <form class="mt-6 space-y-3">
                    <input
                        {oninput}
                        class="w-full rounded-md p-3 border text-gray-800 border-gray-300 bg-white outline-none focus:ring-2 focus:ring-indigo-500"
                        placeholder="Nickname"
                    />
                    <Link<Route> to={Route::Chat}>
                        <button
                            {onclick}
                            disabled={username.trim().len() < 1}
                            class="w-full rounded-md bg-indigo-600 text-white font-semibold p-3 disabled:bg-gray-300 disabled:cursor-not-allowed hover:bg-indigo-700"
                        >
                            {"Join Chat"}
                        </button>
                    </Link<Route>>
                </form>
            </div>
        </div>
    }
}
