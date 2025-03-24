// Basic JavaScript for Navius documentation
document.addEventListener('DOMContentLoaded', () => {
    // Add copy buttons to code blocks
    document.querySelectorAll('pre').forEach(block => {
        const button = document.createElement('button');
        button.textContent = 'Copy';
        button.style.position = 'absolute';
        button.style.right = '5px';
        button.style.top = '5px';
        
        button.addEventListener('click', async () => {
            const code = block.textContent;
            try {
                await navigator.clipboard.writeText(code);
                button.textContent = 'Copied!';
                setTimeout(() => button.textContent = 'Copy', 2000);
            } catch (err) {
                console.error('Failed to copy:', err);
            }
        });
        
        block.style.position = 'relative';
        block.appendChild(button);
    });