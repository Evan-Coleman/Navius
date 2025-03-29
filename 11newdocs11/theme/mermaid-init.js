// Initialize mermaid diagrams
document.addEventListener('DOMContentLoaded', function() {
  mermaid.initialize({
    startOnLoad: true,
    theme: 'neutral',
    flowchart: {
      useMaxWidth: true,
      htmlLabels: true,
      curve: 'basis'
    },
    securityLevel: 'loose',
    fontFamily: '"Noto Sans", "Liberation Sans", Arial, Helvetica, sans-serif'
  });
}); 