<script>
  let title = "Reader";
</script>

<div class="reader-panel">
  <header class="reader-header">
    <h1>{title}</h1>
  </header>

  <section class="reader-body">
    <p>The reader window is ready for note content.</p>
  </section>
</div>

<style>
  .reader-panel {
    height: 100%;
    width: 100%;
    padding: 16px;
    border-radius: var(--border-radius);
    background: rgba(30, 30, 46, 0.55);
    color: var(--text-color, #ffffff);
    backdrop-filter: blur(40px) saturate(180%);
    -webkit-backdrop-filter: blur(40px) saturate(180%);
    border: 0.5px solid rgba(255, 255, 255, 0.14);
    box-shadow: var(--shadow-md);
  }

  .reader-header h1 {
    font-size: 16px;
    font-weight: 600;
  }

  .reader-body {
    margin-top: 12px;
    color: rgba(255, 255, 255, 0.75);
    line-height: 1.5;
  }
</style>
