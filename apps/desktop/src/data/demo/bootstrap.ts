import { configureDataProvider } from "../provider";
import { demoProvider } from "./demo-provider";

/** Explicit entry point for browser demos and component/provider tests. */
export function configureDemoProvider(): void {
  configureDataProvider(demoProvider);
}
