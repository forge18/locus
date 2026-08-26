/* @refresh reload */
import "./workflow-canvas/polyfills";
import { render } from "solid-js/web";
import App from "./App";

render(() => <App />, document.getElementById("root") as HTMLElement);
