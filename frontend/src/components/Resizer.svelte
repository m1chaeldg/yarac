<script lang="ts">
  interface Props {
    direction: 'horizontal' | 'vertical';
    onResize: (delta: number) => void;
    onReset?: () => void;
  }

  let { direction, onResize, onReset }: Props = $props();

  let isDragging = $state(false);
  let startCoord = 0;

  function handleMouseDown(e: MouseEvent) {
    e.preventDefault();
    isDragging = true;
    startCoord = direction === 'horizontal' ? e.clientX : e.clientY;

    const onMouseMove = (moveEvent: MouseEvent) => {
      const currentCoord = direction === 'horizontal' ? moveEvent.clientX : moveEvent.clientY;
      const delta = currentCoord - startCoord;
      startCoord = currentCoord;
      onResize(delta);
    };

    const onMouseUp = () => {
      isDragging = false;
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }
  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      e.preventDefault();
      onResize(-10);
    } else if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
      e.preventDefault();
      onResize(10);
    } else if (e.key === 'Enter' && onReset) {
      e.preventDefault();
      onReset();
    }
  }
</script>

<!-- Resizer divider handle -->
<button
  type="button"
  aria-label="{direction} resize splitter"
  onmousedown={handleMouseDown}
  onkeydown={handleKeyDown}
  ondblclick={onReset}
  class="relative transition-colors select-none z-20 group border-none p-0 outline-none
    {direction === 'horizontal'
      ? 'w-1.5 cursor-col-resize hover:bg-sky-500/50 bg-slate-800/80 active:bg-sky-400'
      : 'h-1.5 cursor-row-resize hover:bg-sky-500/50 bg-slate-800/80 active:bg-sky-400'}"
  class:bg-sky-500={isDragging}
>
  <!-- Centered grip bar -->
  <div
    class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-slate-600 group-hover:bg-sky-300 transition-colors pointer-events-none
      {direction === 'horizontal' ? 'w-0.5 h-6' : 'w-6 h-0.5'}"
  ></div>
</button>
