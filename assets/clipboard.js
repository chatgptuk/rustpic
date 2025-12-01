// Copy to clipboard function
function copyToClipboard(text, buttonId) {
    navigator.clipboard.writeText(text).then(() => {
        showToast('✓ Copied!');
    }).catch(err => {
        showToast('Copy failed', true);
        console.error('Copy failed:', err);
    });
}

// Toast notification
function showToast(message, isError = false) {
    const toast = document.createElement('div');
    toast.className = 'toast' + (isError ? ' toast-error' : '');
    toast.textContent = message;
    document.body.appendChild(toast);

    setTimeout(() => toast.classList.add('show'), 100);
    setTimeout(() => {
        toast.classList.remove('show');
        setTimeout(() => document.body.removeChild(toast), 300);
    }, 2000);
}
