import { Match, Suspense, Switch, lazy, onMount } from "solid-js";
import { Shell } from "./shell/Shell";
import { EmptyPane } from "./ui/EmptyPane";
import { SkeletonRows } from "./ui/SkeletonRows";
import { mountIconSprite } from "./ui/sprite";
import { createNavStore } from "./nav";
import { applyTheme, savedTheme } from "./styles/theme";
import { configureDataProvider, liveProvider } from "./data/provider";
import { searchAll as searchAllResults } from "./data/search";
import { MailView } from "./screens/mail/MailView";
import { AnalyticsView } from "./screens/analytics/AnalyticsView";
import { QAView } from "./screens/review/QAView";
import { DispatchView } from "./screens/dispatch/DispatchView";
import { GuardrailsView } from "./screens/settings/GuardrailsView";
import { WorkersView } from "./screens";
import ManageView from "./screens/manage/ManageView";
import { AgentDefinitionsView } from "./screens/workshop/AgentDefinitionsView";
import { HarnessesView } from "./screens/workshop/HarnessesView";
import { ProvidersView } from "./screens/workshop/ProvidersView";
import { UnavailableWorkshopView } from "./screens/workshop/UnavailableWorkshopView";
import { WorkflowView } from "./screens/workshop/WorkflowView";
import {
        MemoryArtifactsView,
        MemoryLongTermView,
        MemoryShortTermView,
        MemoryWikiView,
} from "./screens/memory/MemoryViews";
import "./styles/app.css";

// The Tauri bootstrap always selects the live provider: a runtime that forgets to
// configure one fails loudly at the first accessor, and demo data is reachable only
// where a host explicitly selects the demo provider (see data/provider.ts).
configureDataProvider(liveProvider);

const InboxView = lazy(() => import("./screens/inbox/InboxView"));
const PlanView = lazy(() => import("./screens/plan/PlanView"));
import { ProjectsView } from "./screens/projects/ProjectsView";
const TelemetryView = lazy(() => import("./screens/review/TelemetryView"));

function App() {
        const nav = createNavStore();
        onMount(() => {
                applyTheme(
                        document.documentElement,
                        savedTheme(window.localStorage),
                );
                mountIconSprite();
        });

        return (
                <Shell
                        nav={nav}
                        searchAll={async (query) => {
                                const envelope = await searchAllResults(query);
                                if (envelope.status === "failed") {
                                        throw new Error(envelope.error.message);
                                }
                                return envelope.status === "ready"
                                        ? envelope.data
                                        : [];
                        }}
                >
                        <Suspense
                                fallback={
                                        <SkeletonRows
                                                count={8}
                                                rowHeight={26}
                                        />
                                }
                        >
                                <Switch
                                        fallback={
                                                <EmptyPane
                                                        icon="signpost"
                                                        reason={`No screen is built for ${nav.view()} yet.`}
                                                />
                                        }
                                >
                                        <Match when={nav.view() === "inbox"}>
                                                <InboxView nav={nav} />
                                        </Match>
                                        <Match when={nav.view() === "status"}>
                                                <AnalyticsView />
                                        </Match>
                                        <Match
                                                when={
                                                        nav.view() ===
                                                        "telemetry"
                                                }
                                        >
                                                <TelemetryView
                                                        projectId={
                                                                nav.params()
                                                                        .project
                                                        }
                                                />
                                        </Match>
                                        <Match when={nav.view() === "projects"}>
                                                <ProjectsView />
                                        </Match>
                                        <Match when={nav.view() === "plan"}>
                                                <PlanView nav={nav} />
                                        </Match>
                                        <Match when={nav.view() === "sessions"}>
                                                <ManageView
                                                        projectId={
                                                                nav.params()
                                                                        .project
                                                        }
                                                />
                                        </Match>
                                        <Match when={nav.view() === "runs"}>
                                                <DispatchView
                                                        tab="runs"
                                                        nav={nav}
                                                />
                                        </Match>
                                        <Match when={nav.view() === "artifact"}>
                                                <MemoryArtifactsView
                                                        projectId={
                                                                nav.params()
                                                                        .project
                                                        }
                                                />
                                        </Match>
                                        <Match when={nav.view() === "wiki"}>
                                                <MemoryWikiView />
                                        </Match>
                                        <Match when={nav.view() === "agents"}>
                                                <AgentDefinitionsView />
                                        </Match>
                                        <Match
                                                when={
                                                        nav.view() ===
                                                        "harnesses"
                                                }
                                        >
                                                <HarnessesView />
                                        </Match>
                                        <Match when={nav.view() === "mail"}>
                                                <MailView />
                                        </Match>
                                        <Match when={nav.view() === "workers"}>
                                                <WorkersView
                                                        projectId={
                                                                nav.params()
                                                                        .project
                                                        }
                                                        botId={
                                                                nav.params()
                                                                        .botId
                                                        }
                                                />
                                        </Match>
                                        <Match when={nav.view() === "qa"}>
                                                <QAView
                                                        projectId={
                                                                nav.params()
                                                                        .project
                                                        }
                                                />
                                        </Match>
                                        <Match when={nav.view() === "autorun"}>
                                                <DispatchView
                                                        tab="autorun"
                                                        nav={nav}
                                                />
                                        </Match>
                                        <Match when={nav.view() === "schedule"}>
                                                <DispatchView
                                                        tab="schedules"
                                                        nav={nav}
                                                />
                                        </Match>
                                        <Match when={nav.view() === "short"}>
                                                <MemoryShortTermView />
                                        </Match>
                                        <Match when={nav.view() === "memory"}>
                                                <MemoryLongTermView
                                                        projectId={
                                                                nav.params()
                                                                        .project
                                                        }
                                                />
                                        </Match>
                                        <Match when={nav.view() === "settings"}>
                                                <GuardrailsView />
                                        </Match>
                                        <Match when={nav.view() === "cli"}>
                                                <UnavailableWorkshopView
                                                        route="cli"
                                                        label="CLI tools"
                                                        command="extension_inventory"
                                                />
                                        </Match>
                                        <Match when={nav.view() === "commands"}>
                                                <UnavailableWorkshopView
                                                        route="commands"
                                                        label="Commands"
                                                        command="extension_inventory"
                                                />
                                        </Match>
                                        <Match when={nav.view() === "hooks"}>
                                                <UnavailableWorkshopView
                                                        route="hooks"
                                                        label="Hooks"
                                                        command="extension_inventory"
                                                />
                                        </Match>
                                        <Match when={nav.view() === "linters"}>
                                                <UnavailableWorkshopView
                                                        route="linters"
                                                        label="Linters"
                                                        command="extension_inventory"
                                                />
                                        </Match>
                                        <Match when={nav.view() === "styles"}>
                                                <UnavailableWorkshopView
                                                        route="styles"
                                                        label="Output styles"
                                                        command="extension_inventory"
                                                />
                                        </Match>
                                        <Match
                                                when={
                                                        nav.view() ===
                                                        "providers"
                                                }
                                        >
                                                <ProvidersView />
                                        </Match>
                                        <Match when={nav.view() === "rules"}>
                                                <UnavailableWorkshopView
                                                        route="rules"
                                                        label="Rules"
                                                        command="extension_inventory"
                                                />
                                        </Match>
                                        <Match when={nav.view() === "skills"}>
                                                <UnavailableWorkshopView
                                                        route="skills"
                                                        label="Skills"
                                                        command="extension_inventory"
                                                />
                                        </Match>
                                        <Match when={nav.view() === "canvas"}>
                                                <WorkflowView
                                                        projectId={
                                                                nav.params()
                                                                        .project
                                                        }
                                                />
                                        </Match>
                                        <Match
                                                when={
                                                        nav.view() ===
                                                        "workflows"
                                                }
                                        >
                                                <WorkflowView
                                                        projectId={
                                                                nav.params()
                                                                        .project
                                                        }
                                                />
                                        </Match>
                                </Switch>
                        </Suspense>
                </Shell>
        );
}

export default App;
