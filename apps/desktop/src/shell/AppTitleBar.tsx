export function AppTitleBar() {
  return (
    <div class="titlebar" data-testid="app-titlebar">
      <div class="traffic" data-testid="traffic-lights">
        <span class="traffic-close" />
        <span class="traffic-min" />
        <span class="traffic-max" />
      </div>
      <div class="wordmark" data-testid="wordmark">
        Locus
      </div>
    </div>
  )
}
