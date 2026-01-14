<div class="view-container">
  <h1>Settings</h1>
  
  <div class="section">
    <h2>General</h2>
    <div class="setting">
      <label>
        <input type="checkbox" checked />
        Start with Windows
      </label>
    </div>
  </div>

  <div class="section">
    <h2>Audio</h2>
    <div class="setting">
      <label>Output Device</label>
      <select>
        <option>Default Output</option>
        <option>External DAC</option>
      </select>
    </div>
  </div>

  <div class="section">
    <h2>Theme</h2>
    <div class="setting">
      <label>Mode</label>
      <select disabled>
        <option>Dark (Default)</option>
      </select>
    </div>
  </div>
</div>

<style>
  .view-container {
    padding: 2rem;
    color: #fff;
    height: 100%;
    overflow-y: auto;
  }
  .section {
    margin-bottom: 2rem;
  }
  h2 {
    font-size: 1.2rem;
    border-bottom: 1px solid #333;
    padding-bottom: 0.5rem;
    margin-bottom: 1rem;
    color: #ccc;
  }
  .setting {
    margin-bottom: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  select {
    background: #222;
    color: #fff;
    border: 1px solid #333;
    padding: 0.5rem;
    border-radius: 4px;
    width: 200px;
  }
</style>
