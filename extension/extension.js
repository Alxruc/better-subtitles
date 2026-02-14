const transcribeButton = document.getElementById('transcribe');
const getSubtitlesButton = document.getElementById('getSubtitles')
const statusText = document.getElementById('statusText');
const preview = document.getElementById('preview');
const stylingButton = document.getElementById('styling');

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
  statusText.textContent = text;
  statusText.style.color = isError ? '#f04040ff' : '';
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

async function getAndCheckYTurl() {
  showStatus('Checking active tab…');
  preview.style.display = 'none';
  const url = await getActiveTabUrl();

  if (!url) {
      showStatus('Unable to determine the current tab URL. Make sure the extension has permission to read tabs.', true);
      return "";
  }

  if (!isYouTubeUrl(url)) {
      showStatus('Current tab is not a YouTube URL. Open a YouTube video page and try again.', true);
      preview.textContent = url;
      preview.style.display = 'block';
      return "";
  }

  return url;
}


getSubtitlesButton.addEventListener('click', async () => {
  try {
    // Send the message to that specific tab
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    const response = await chrome.tabs.sendMessage(tab.id, 'better-subtitles-display');

    if (response?.statusCode !== 200) {
       showStatus("Error displaying subtitles.", true);
    }
  } catch (error) {
    // This catches "Receiving end does not exist" if content script isn't loaded
    console.error(error);
    showStatus("Please reload the YouTube page and try again.", true);
  }
})

transcribeButton.addEventListener('click', async () => {
    const url = await getAndCheckYTurl();
    const deep = 'better-subtitles://save?url=' + encodeURIComponent(url);
    preview.textContent = deep;
    preview.style.display = 'block';
    showStatus('Opening Better Subtitles…');

    // small delay to allow user to see preview/status in popup UI
    setTimeout(() => openDeepLink(deep), 220);
});

stylingButton.addEventListener('click', () => {
  if (chrome.runtime.openOptionsPage) {
    chrome.runtime.openOptionsPage();
  } else {
    window.open(chrome.runtime.getURL('settings.html'));
  }
});