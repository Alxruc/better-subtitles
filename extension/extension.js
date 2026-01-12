const btn = document.getElementById('transcribe');
const status = document.getElementById('status');
const preview = document.getElementById('preview');

function isYouTubeUrl(u) {
    if (!u) return false;
    // match youtube watch, youtu.be short links, and /shorts/
    const re = /^(?:https?:\/\/)?(?:www\.)?(?:youtube\.com\/(?:watch\?v=|shorts\/)|youtu\.be\/)/i;
    return re.test(u);
}

function openDeepLink(target) {
    // Try navigation first, then fallback to clickable anchor (for better compatibility)
    try {
        window.location.href = target;
    } catch (e) {
        const a = document.createElement('a');
        a.href = target;
        a.style.display = 'none';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
    }
}

function showStatus(text, isError) {
    status.textContent = text;
    status.style.color = isError ? '#ffb4b4' : '';
}

async function getActiveTabUrl() {
    // Prefer chrome.tabs API when available (extension popup). Fallback to document.referrer/location.
    // Works in Chrome/Edge/Firefox (browser) if extension has required permissions.
    try {
        if (typeof chrome !== 'undefined' && chrome.tabs && chrome.tabs.query) {
            return new Promise((resolve) => {
                chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
                    if (tabs && tabs[0] && tabs[0].url) resolve(tabs[0].url);
                    else resolve(null);
                });
            });
        }
        // Fallback: the popup itself won't have the page URL, but sometimes document.referrer contains it.
        if (document.referrer) return document.referrer;
        return window.location.href || null;
    } catch (e) {
        return null;
    }
}

btn.addEventListener('click', async () => {
    showStatus('Checking active tab…');
    preview.style.display = 'none';
    const url = await getActiveTabUrl();

    if (!url) {
        showStatus('Unable to determine the current tab URL. Make sure the extension has permission to read tabs.', true);
        return;
    }

    if (!isYouTubeUrl(url)) {
        showStatus('Current tab is not a YouTube URL. Open a YouTube video page and try again.', true);
        preview.textContent = url;
        preview.style.display = 'block';
        return;
    }

    const deep = 'better-subtitles://save?url=' + encodeURIComponent(url);
    preview.textContent = deep;
    preview.style.display = 'block';
    showStatus('Opening Better Subtitles…');

    // small delay to allow user to see preview/status in popup UI
    setTimeout(() => openDeepLink(deep), 220);
});
