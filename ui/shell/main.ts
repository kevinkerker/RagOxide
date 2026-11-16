import { mount } from "svelte";
import { initBackend } from "$api";
import App from "./App.svelte";
import "./tokens.css";
import "./base.css";

initBackend();
mount(App, { target: document.getElementById("app")! });
