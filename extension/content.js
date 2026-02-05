let transcript = [];
let currentIndex = 0;
let subtitleOverlay = null;

let lastUrl = location.href; 
// youtube is a single page application so we dont want to display
// wrong subtitles when switching videos
new MutationObserver(() => {
  const url = location.href;
  if (url !== lastUrl) {
    lastUrl = url;
    // URL changed! Clear old data and fetch new.
    transcript = [];
    if(subtitleOverlay) subtitleOverlay.innerText = "";
  }
}).observe(document, {subtree: true, childList: true});

function ensureOverlay() {
    if (document.getElementById('bt-subtitle-layer')) return;
    
    const container = document.getElementById('movie_player');
    if (!container) return 500;

    subtitleOverlay = document.createElement('div');
    subtitleOverlay.id = 'bt-subtitle-layer';

    subtitleOverlay.textContent = "Better Subtitles Initialized";
    
    // styles here to rule out CSS file issues
    Object.assign(subtitleOverlay.style, {
        position: 'absolute',
        bottom: '50px',
        left: '50%',
        transform: 'translateX(-50%)',
        fontSize: '24px',
        fontWeight: 'bold',
        color: 'white',
        background: '#04203338',     // Bright background to make it obvious
        zIndex: '99999999',    // Maximum Z-index
    });
    
    container.appendChild(subtitleOverlay);
    return 200;
}

async function tryFetchTranscript() {
    // Current YouTube URL
    const currentUrl = window.location.href;
    
    try {
        // Fetch from application
        const res = await fetch(`http://127.0.0.1:14567/subtitles?url=${encodeURIComponent(currentUrl)}`);
        const data = await res.json();
        
        if (data.error) {
            console.log("No transcript found locally.");
            return 404;
        }

        transcript = data;
        console.log(transcript);
        currentIndex = 0;
        return ensureOverlay();
    } catch (e) {
        // App probably not running
        console.debug("Better Subtitles app not detected.");
        return 500;
    }
}


function onTimeUpdate(e) {
    if (!transcript.length || !subtitleOverlay) return;

    const time = e.target.currentTime;
    
    // If the user seeks backwards, reset index
    if (currentIndex > 0 && transcript[currentIndex].start > time) {
        currentIndex = 0;
    }

    // Advance index until we find the segment that ends AFTER the current time
    while (currentIndex < transcript.length - 1) {
        if (transcript[currentIndex].start >= time) {
            currentIndex = currentIndex == 0 ? 0 : currentIndex - 1;
            break;
        }
        currentIndex++
    }

    const segment = transcript[currentIndex];

    subtitleOverlay.textContent = segment.text;
}

async function displaySubtitles() {
  const videoContainer = document.querySelector(".ytd-player")

  if (!videoContainer) {
    return 500;
  }

  const video = document.querySelector('video');
  if (video) {
      video.addEventListener('timeupdate', onTimeUpdate);
      // Also listen for seek events to clear text immediately
      video.addEventListener('seeking', () => { subtitleOverlay.innerText = ""; });
  }
  return tryFetchTranscript();
}

function handleMessages(message, sender, sendResponse) {
  if (message !== 'better-subtitles-display') {
    console.log("random message ", message)
    return;
  } 
  displaySubtitles().then((response) => sendResponse({statusCode: response}));

  return true; // because async return explicit true
}

chrome.runtime.onMessage.addListener(handleMessages);