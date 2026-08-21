import { For, Show, createMemo, createSignal } from 'solid-js'
import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import { Textarea } from '../../ui/Input'
import { Resizable } from '../../panes/Resizable'
import {
  ARTIFACT_LOCATOR,
  COMMENTS_TITLE,
  LIVE_COMMENT_NOTE,
  ONE_VIEWER_NOTE,
  REFERENCE_GROUP_LABEL,
  RESOLVE,
  SEND_TO_SESSION,
  useArtifactComments,
  useArtifactKinds,
  useArtifacts,
  useDefaultArtifactId,
  useUnifiedDiff,
} from '../../data/artifacts'

export interface ArtifactsViewProps {
  /** Which artifact to open. The same viewer whichever entry point set it. */
  artifactId?: string
}

/**
 * One viewer per kind, several entry points. An artifact reached from the inbox,
 * from the board, or from a run renders the same — a diff that looks different
 * depending on how you got to it is two components that will disagree.
 */
export function ArtifactsView(props: ArtifactsViewProps) {
  const kinds = useArtifactKinds()
  const [selectedId, setSelectedId] = createSignal(props.artifactId ?? useDefaultArtifactId())
  const artifact = createMemo(
    () => useArtifacts().find((a) => a.id === selectedId()) ?? useArtifacts()[0],
  )

  return (
    <div class="artifacts" data-testid="artifacts">
      <Resizable width={222} min={180} max={360} side="right" class="artifact-list" testId="artifact-list">
        <div class="artifact-list-body">
          <div class="artifact-group" data-testid="artifact-group-review">
            Review artifacts
          </div>
          <For each={kinds.review}>
            {(entry, i) => (
              <button
                type="button"
                class="artifact-entry"
                data-testid={`artifact-entry-${entry.label}`}
                data-group="review"
                aria-selected={i() === 0 && selectedId() === 'a-1' ? 'true' : 'false'}
                onClick={() => setSelectedId('a-1')}
              >
                <Icon name={entry.icon} size={11} />
                {entry.label}
                <span class="artifact-entry-note">{entry.note}</span>
              </button>
            )}
          </For>

          <div
            class="artifact-group artifact-group-reference"
            data-testid="artifact-group-reference"
          >
            {REFERENCE_GROUP_LABEL}
          </div>
          <For each={kinds.reference}>
            {(entry) => (
              <div
                class="artifact-entry artifact-entry-reference"
                data-testid={`artifact-entry-${entry.label}`}
                data-group="reference"
              >
                <Icon name={entry.icon} size={11} />
                {entry.label}
                <span class="artifact-entry-note">{entry.note}</span>
              </div>
            )}
          </For>
        </div>
      </Resizable>

      <section class="artifact-view" data-testid="artifact-view">
        <header class="artifact-head" data-testid="artifact-head">
          <span class="wiki-kind" data-testid="artifact-kind">
            {artifact().kind}
          </span>
          <span class="artifact-name" data-testid="artifact-name">
            {artifact().title}
          </span>
          <span class="artifact-locator" data-testid="artifact-locator">
            {ARTIFACT_LOCATOR}
          </span>
          <span class="artifact-note" data-testid="artifact-one-viewer-note">
            {ONE_VIEWER_NOTE}
          </span>
        </header>

        <div class="udiff" data-testid="udiff">
          <For each={useUnifiedDiff()}>
            {(row) => (
              <div
                class={[
                  'udiff-row',
                  row.kind === 'hunk' ? 'udiff-hunk' : '',
                  row.kind === 'added' ? 'udiff-added' : '',
                  row.kind === 'removed' ? 'udiff-removed' : '',
                  row.commented ? 'udiff-commented' : '',
                ]
                  .filter(Boolean)
                  .join(' ')}
                data-kind={row.kind}
                data-commented={row.commented ? 'true' : undefined}
                data-testid={row.kind === 'hunk' ? `udiff-hunk-${row.text.split(' ')[1]}` : undefined}
              >
                <span class="udiff-gutter">{row.no ?? ''}</span>
                <span class="udiff-text">{row.text}</span>
              </div>
            )}
          </For>
        </div>
      </section>

      <Resizable width={306} min={240} max={420} side="left" class="comment-rail" testId="comment-rail">
        <div class="artifact-group" style={{ padding: '11px 11px 0' }} data-testid="comments-title">
          {COMMENTS_TITLE}
        </div>
        <div class="comment-rail-body">
          <For each={useArtifactComments(artifact().id)}>
            {(comment) => (
              <div
                class={['comment', comment.author !== 'you' ? 'comment-agent' : '']
                  .filter(Boolean)
                  .join(' ')}
                data-testid={`comment-${comment.id}`}
                data-author={comment.author}
              >
                <span class="comment-avatar">
                  {comment.author === 'you' ? 'YOU' : comment.author.slice(0, 2).toUpperCase()}
                </span>
                <span class="comment-body">{comment.body}</span>
              </div>
            )}
          </For>
          <div class="comment-live" data-testid="comment-live">
            <span class="live-dot pulse" data-testid="comment-live-dot" />
            {LIVE_COMMENT_NOTE}
          </div>
        </div>
        <footer class="comment-foot" data-testid="comment-foot">
          <Textarea data-testid="comment-input" placeholder="Comment on the marked line" />
          <div class="comment-foot-actions">
            <Button variant="primary" data-testid="comment-send">
              {SEND_TO_SESSION}
            </Button>
            <Button variant="secondary" data-testid="comment-resolve">
              {RESOLVE}
            </Button>
          </div>
        </footer>
      </Resizable>

      <Show when={false}>
        <span />
      </Show>
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default ArtifactsView
