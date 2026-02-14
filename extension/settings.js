const defaultSettings = {
    textColor: "#ffffff",
    fontSize: "24",
    bgColor: "#042033",
    bgOpacity: "0.8",
    bottomOffset: "50"
};

// Elements
const inputs = {
    textColor: document.getElementById('textColor'),
    fontSize: document.getElementById('fontSize'),
    bgColor: document.getElementById('bgColor'),
    bgOpacity: document.getElementById('bgOpacity'),
    bottomOffset: document.getElementById('bottomOffset'),
    clickThrough: document.getElementById('clickThrough')
};
const previewEl = document.getElementById('bt-subtitle-layer');
const saveBtn = document.getElementById('saveBtn');
const statusEl = document.getElementById('status');

// Load saved settings
document.addEventListener('DOMContentLoaded', () => {
    chrome.storage.local.get(['better-subtitles-settings'], (result) => {
        const settings = result['better-subtitles-settings'] || defaultSettings;
        
        // Populate inputs
        Object.keys(inputs).forEach(key => {
            if (settings[key]) inputs[key].value = settings[key];
        });
        updatePreview();
    });
});

// Update preview on any input change
Object.values(inputs).forEach(input => {
    input.addEventListener('input', updatePreview);
});

function hexToRgba(hex, alpha) {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function updatePreview() {
    previewEl.style.color = inputs.textColor.value;
    previewEl.style.fontSize = inputs.fontSize.value + 'px';
    previewEl.style.backgroundColor = hexToRgba(inputs.bgColor.value, inputs.bgOpacity.value);
    previewEl.style.bottom = inputs.bottomOffset.value + 'px';
    
    previewEl.style.position = 'absolute';
    if (inputs.clickThrough.checked) {
        previewEl.style.pointerEvents = 'none';
        previewEl.style.border = '1px dashed #666';
        previewEl.title = "Clicks pass through";
    } else {
        previewEl.style.pointerEvents = 'auto';
        previewEl.style.border = '1px solid white';
        previewEl.title = "Text is selectable";
    }
}

// Save Settings
saveBtn.addEventListener('click', () => {
    const newSettings = {
        textColor: inputs.textColor.value,
        fontSize: inputs.fontSize.value,
        bgColor: inputs.bgColor.value,
        bgOpacity: inputs.bgOpacity.value,
        bottomOffset: inputs.bottomOffset.value,
    };

    chrome.storage.local.set({ 'better-subtitles-settings': newSettings }, () => {
        statusEl.textContent = "Settings saved!";
        setTimeout(() => statusEl.textContent = "", 2000);
    });
});