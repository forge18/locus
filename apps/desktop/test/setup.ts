import { configureDemoProvider } from "../src/data/demo/bootstrap";

// Component/provider tests opt into demo data explicitly. Production App.tsx
// configures liveProvider instead and never imports this bootstrap.
configureDemoProvider();
configureDemoProvider();
