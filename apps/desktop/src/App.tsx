import { Match, Suspense, Switch, lazy, onMount } from "solid-js";
import { Shell } from "./shell/Shell";
import { EmptyPane } from "./ui/EmptyPane";
import { SkeletonRows } from "./ui/SkeletonRows";
import { mountIconSprite } from "./ui/sprite";
import { createNavStore } from "./nav";
import { applyTheme, savedTheme } from "./styles/theme";
import { MailView } from "./screens/mail/MailView";
import { AnalyticsView } from "./screens/analytics/AnalyticsView";
import { QAView } from "./screens/review/QAView";
import { DispatchView } from "./screens/dispatch/DispatchView";
import { GuardrailsView } from "./screens/settings/GuardrailsView";
import InteractView from "./screens/interact/InteractView";
import ManageView from "./screens/manage/ManageView";
import WorkshopFixtureView from "./screens/workshop/WorkshopFixtureView";
import {
        MemoryArtifactsFixture,
        MemoryLongTermFixture,
        MemoryShortTermFixture,
        MemoryWikiFixture,
} from "./screens/memory/MemoryFixtures";
import "./styles/app.css";

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
                <Shell nav={nav}>
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
                                                <TelemetryView />
                                        </Match>
                                        <Match when={nav.view() === "projects"}>
                                                <ProjectsView />
                                        </Match>
                                        <Match when={nav.view() === "plan"}>
                                                <PlanView />
                                        </Match>
                                        <Match when={nav.view() === "sessions"}>
                                                <ManageView />
                                        </Match>
                                        <Match when={nav.view() === "runs"}>
                                                <DispatchView tab="runs" />
                                        </Match>
                                        <Match when={nav.view() === "artifact"}>
                                                <MemoryArtifactsFixture />
                                        </Match>
                                        <Match when={nav.view() === "wiki"}>
                                                <MemoryWikiFixture />
                                        </Match>
                                        <Match when={nav.view() === "agents"}>
                                                <WorkshopFixtureView fixture="agents" />
                                        </Match>
                                        <Match
                                                when={
                                                        nav.view() ===
                                                        "harnesses"
                                                }
                                        >
                                                <WorkshopFixtureView fixture="harnesses" />
                                        </Match>
                                        <Match when={nav.view() === "mail"}>
                                                <MailView />
                                        </Match>
                                        <Match when={nav.view() === "interact"}>
                                                <InteractView />
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
                                                <DispatchView tab="autorun" />
                                        </Match>
                                        <Match when={nav.view() === "schedule"}>
                                                <DispatchView tab="schedules" />
                                        </Match>
                                        <Match when={nav.view() === "short"}>
                                                <MemoryShortTermFixture />
                                        </Match>
                                        <Match when={nav.view() === "memory"}>
                                                <MemoryLongTermFixture />
                                        </Match>
                                        <Match when={nav.view() === "settings"}>
                                                <GuardrailsView />
                                        </Match>
                                        <Match when={nav.view() === "cli"}>
                                                <WorkshopFixtureView fixture="cli" />
                                        </Match>
                                        <Match when={nav.view() === "commands"}>
                                                <WorkshopFixtureView fixture="commands" />
                                        </Match>
                                        <Match when={nav.view() === "hooks"}>
                                                <WorkshopFixtureView fixture="hooks" />
                                        </Match>
                                        <Match when={nav.view() === "linters"}>
                                                <WorkshopFixtureView fixture="linters" />
                                        </Match>
                                        <Match when={nav.view() === "styles"}>
                                                <WorkshopFixtureView fixture="styles" />
                                        </Match>
                                        <Match
                                                when={
                                                        nav.view() ===
                                                        "providers"
                                                }
                                        >
                                                <WorkshopFixtureView fixture="providers" />
                                        </Match>
                                        <Match when={nav.view() === "rules"}>
                                                <WorkshopFixtureView fixture="rules" />
                                        </Match>
                                        <Match when={nav.view() === "skills"}>
                                                <WorkshopFixtureView fixture="skills" />
                                        </Match>
                                        <Match when={nav.view() === "canvas"}>
                                                <WorkshopFixtureView fixture="workflows-visual" />
                                        </Match>
                                        <Match
                                                when={
                                                        nav.view() ===
                                                        "workflows"
                                                }
                                        >
                                                <WorkshopFixtureView fixture="workflows-list" />
                                        </Match>
                                </Switch>
                        </Suspense>
                </Shell>
        );
}

export default App;
