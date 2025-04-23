// principal.js
// Wait for the HTML document to be fully loaded before running the script
document.addEventListener("DOMContentLoaded", () => {

  // --- 1. DOM Element References ---
  // Tabs
  const tabIndicator = document.getElementById('tabIndicator');
  const tabButtons = document.querySelectorAll('.tab-button');
  const tabContents = document.querySelectorAll('.tab-content');

  // Text Tab Elements
  const inputText = document.getElementById('inputText');       // Textarea
  const outputText = document.getElementById('outputText');     // Output display
  const textConvertButton = document.getElementById('textConvertButton'); // Convert button
  const textSpeakButton = document.getElementById('textSpeakButton');     // Speak button
  const textDownloadButton = document.getElementById('textDownloadButton'); // Download button

  // File Tab Elements
  const dropzone = document.getElementById('dropzone');         // Drop area
  const fileInput = document.getElementById('file-input');      // Hidden file input
  const fileList = document.getElementById('file-list');        // List for file names
  const fileOutput = document.getElementById('fileOutput');     // Output display
  const fileConvertButton = document.getElementById('fileConvertButton'); // Convert button
  const fileSpeakButton = document.getElementById('fileSpeakButton');     // Speak button
  const fileDownloadButton = document.getElementById('fileDownloadButton'); // Download button


  // --- Safety Check (Optional but good practice) ---
  if (!tabIndicator || tabButtons.length === 0 || tabContents.length === 0 || !dropzone || !fileInput || !fileList) {
      console.error("Essential UI elements missing. Check HTML IDs and classes.");
      // return; // Halt execution if critical elements are missing
  }

  // --- 2. Helper Function: Tab Switching ---
  /**
   * Shows a specific tab and hides others. Updates the visual indicator.
   * @param {string} tabId - The ID of the tab content element to show.
   * @param {number} index - The index (0-based) of the clicked tab button.
   */
  function showTab(tabId, index) {
    // Ensure elements exist before manipulating
    if (!tabIndicator || !tabContents.length || !tabButtons.length) return;

    // Hide all tab content sections
    tabContents.forEach(tab => tab?.classList.remove('active')); // Add null check

    // Show the selected tab content section
    const selectedTabContent = document.getElementById(tabId);
    if (selectedTabContent) {
        selectedTabContent.classList.add('active');
    } else {
        console.error(`Tab content with ID '${tabId}' not found.`);
        return; // Exit if the target tab content doesn't exist
    }


    // Move the visual indicator under the active tab button
    if (tabIndicator) { // check if tabIndicator exists
        const buttonWidth = tabButtons[index]?.offsetWidth || 100; // Default width if offsetWidth is not available
        tabIndicator.style.width = `${buttonWidth}px`;
        tabIndicator.style.transform = `translateX(${index * buttonWidth}px)`;

    }

    // Update the active state styling for the tab buttons
    tabButtons.forEach((button, i) => {
      button?.classList.toggle('active', i === index); // Add null check
    });
  }

  // --- 3. Helper Function: Handle File Selection (Minimal: List names only) ---
  /**
   * Lists the names of selected files. Does NOT read content.
   * @param {FileList} files - The list of files from the input or drop event.
   */
  function handleFiles(files) {
    if (!files || files.length === 0 || !fileList) return; // Exit if no files or list element missing

    fileList.innerHTML = ''; // Clear the previous list of file names

    // List all selected file names
    Array.from(files).forEach(f => {
      const li = document.createElement('li');
      li.textContent = f.name; // Display the name
      fileList.appendChild(li);
    });

    // Optionally, clear or update the fileOutput simulation area
    fileOutput.textContent = `${files.length} file(s) selected.`; // simplified

  }

  // --- 4. Tab Switching Logic ---
  tabButtons.forEach((button, index) => {
    if (button) { // Check if button exists
      button.addEventListener('click', () => {
        const tabId = button.dataset.tab; // Get the target tab ID
        if (tabId) {
            showTab(tabId, index);            // Show the corresponding tab
        } else {
            console.error("Tab button missing 'data-tab' attribute:", button);
        }
      });
    }
  });

  // Show the first tab ('text') by default when the page loads
  if (tabButtons.length > 0 && tabButtons[0].dataset.tab) {
    showTab(tabButtons[0].dataset.tab, 0);
  } else if (tabContents.length > 0) {
    // Fallback: Show the first content if buttons are misconfigured
    showTab(tabContents[0].id, 0);
  }


  // --- 5. Dropzone Setup (Visuals and File Selection Trigger) ---
  if (dropzone && fileInput) {
    // Make the dropzone clickable
    dropzone.addEventListener('click', (e) => {
      e.preventDefault();
      fileInput.click(); // Trigger the hidden file input
    });

    // Prevent default drag/drop browser behavior
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
      dropzone.addEventListener(eventName, (e) => {
        e.preventDefault();
        e.stopPropagation();
      }, false);
    });

    // Add/Remove visual feedback class
    ['dragenter', 'dragover'].forEach(eventName => {
      dropzone.addEventListener(eventName, () => {
        dropzone.classList.add('dragover-highlight');
      }, false);
    });
    ['dragleave', 'drop'].forEach(eventName => {
      dropzone.addEventListener(eventName, () => {
        dropzone.classList.remove('dragover-highlight');
      }, false);
    });

    // Handle dropped files (call minimal handler)
    dropzone.addEventListener('drop', (e) => {
      const files = e.dataTransfer?.files; // Use optional chaining
      if (files) {
          handleFiles(files);
      }
    });
  } // End if (dropzone && fileInput)

  // Handle files selected via the input field (call minimal handler)
  if (fileInput) {
    fileInput.addEventListener('change', () => {
      if (fileInput.files) {
        handleFiles(fileInput.files);
      }
    });
  }


  // --- 6. Button Event Listeners (No Actions) ---

  // Text Tab Buttons
  if (textConvertButton) {
    textConvertButton.addEventListener('click', () => {
      console.log("Text Convert Button Clicked (No Action)");
      // You could update outputText for simulation:
       outputText.textContent = `Simulating conversion for: ${inputText.value.substring(0, 50)}...`; // Fixed to show output text
    });
  }
  if (textSpeakButton) {
    textSpeakButton.addEventListener('click', () => {
      console.log("Text Speak Button Clicked (No Action)");
    });
  }
  if (textDownloadButton) {
    textDownloadButton.addEventListener('click', () => {
      console.log("Text Download Button Clicked (No Action)");
    });
  }

  // File Tab Buttons
  if (fileConvertButton) {
    fileConvertButton.addEventListener('click', () => {
      console.log("File Convert Button Clicked (No Action)");
       // You could update fileOutput for simulation:
      fileOutput.textContent = `Simulating conversion for selected files...`; // Fixed for file output
    });
  }
  if (fileSpeakButton) {
    fileSpeakButton.addEventListener('click', () => {
      console.log("File Speak Button Clicked (No Action)");
    });
  }
  if (fileDownloadButton) {
    fileDownloadButton.addEventListener('click', () => {
      console.log("File Download Button Clicked (No Action)");
    });
  }
  // --- 7. Function Definitions (missing from original)---
  // You'll need to define these placeholder functions.  They will be called from the HTML.

  window.convertText = function() {
    // Implement text conversion logic here
    console.log("convertText function called");
    outputText.textContent = "Text conversion in progress..."; // example
  };

  window.speakText = function() {
    // Implement text-to-speech logic here
    console.log("speakText function called");
  };

  window.downloadText = function() {
    // Implement text download logic here
    console.log("downloadText function called");
  };

  window.convertFile = function() {
    // Implement file conversion logic here
    console.log("convertFile function called");
    fileOutput.textContent = "File conversion in progress..."; //example
  };

  window.speakFile = function() {
    // Implement file-to-speech logic here
    console.log("speakFile function called");
  };

  window.downloadFile = function() {
    // Implement file download logic here
    console.log("downloadFile function called");
  };


}); // End of DOMContentLoaded listener