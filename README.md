## Experiment 3.1: Original Code

![Chatbox Image](images/Chatbox.png)

![Register Image](images/Register.png)

## Experiment 3.2: Creative Improvements

![Updated Chatbox Image](images/Updated_Chatbox.png)

![Register Image](images/Updared_Register.png)

The register page was simplified into a clean centered card with a nickname input and a Join Chat button. This makes the first screen easier to understand before entering the chat room.

In the chat page, I added emoji buttons so users can quickly use the emojis for their message. I also added a timestamp beside each sender name so every message shows when it was sent.

I also made two small behavior improvements, empty messages are not sent, and the sidebar now shows the number of online users with a label such as Users (2).

## Bonus: Rust Websocket Server for yew Chat

For the bonus task, I added a Rust websocket server in "rust-server/" to replace the JavaScript websocket server from Tutorial 3. The Yew client still connects to the same endpoint, ws://127.0.0.1:8080, so no client-side websocket URL change is needed.

The main difference is that the Rust server now understands the JSON text messages used by the Yew client. Even though the websocket payload is sent as text, the text contains serialized JSON. The Rust server deserializes incoming JSON messages such as `register` and `message`, updates the connected user list, and serializes JSON responses back to every connected client.

This change is successful because the browser client can keep using the same Yew websocket service while the backend is now implemented in Rust. I prefer the Rust version for this project because it gives stronger type checking for the websocket message format.
