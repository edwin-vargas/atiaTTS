  // Wait for the HTML document to be fully loaded before running the script
  document.addEventListener("DOMContentLoaded", () => {

    
    // --- 1. DOM Element References ---
    // Get references to the HTML elements needed for tab functionality
    const tabIndicator = document.getElementById('tabIndicator');    // The visual indicator showing the active tab
    const tabButtons = document.querySelectorAll('.tab-button');    // All buttons used to switch tabs
    const tabContents = document.querySelectorAll('.tab-content');  // All content areas corresponding to the tabs

    // Get references for the "Text" tab functionality
    const inputText = document.getElementById('inputText');          // The textarea where the user types text
    const outputText = document.getElementById('outputText');        // An area to display the text
    // Button in the FIRST panel of the TEXT tab
    const textProcessButton = document.querySelector('#text .panel:nth-child(1) .controls button:nth-child(1)'); // "Convertir" button
    // Buttons in the SECOND panel of the TEXT tab
    const textSpeakButton = document.querySelector('#text .panel:nth-child(2) .controls button:nth-child(1)');   // "Reproducir" button
    const textDownloadButton = document.querySelector('#text .panel:nth-child(2) .controls button:nth-child(2)'); // "Descargar" button

    // Get references for the "File" tab functionality
    const dropzone = document.getElementById('dropzone');            // The area where files can be dropped
    const fileInput = document.getElementById('file-input');        // The hidden input field for manual file selection
    const fileList = document.getElementById('file-list');          // An area to list the names of uploaded files
    const fileOutput = document.getElementById('fileOutput');        // An area to display the content of the uploaded file
    // Button in the FIRST panel of the FILE tab
    const fileProcessButton = document.querySelector('#file .panel:nth-child(1) .controls button:nth-child(1)'); // "Convertir" button (file)
    // Buttons in the SECOND panel of the FILE tab
    const fileSpeakButton = document.querySelector('#file .panel:nth-child(2) .controls button:nth-child(1)');   // "Reproducir" button (file)
    const fileDownloadButton = document.querySelector('#file .panel:nth-child(2) .controls button:nth-child(2)'); // "Descargar" button (file)

    // --- Safety Checks ---
    if (!textProcessButton || !textSpeakButton || !textDownloadButton || !fileProcessButton || !fileSpeakButton || !fileDownloadButton) {
        console.error("Error: Could not find one or more control buttons. Check HTML structure and JS selectors.");
        // Optionally, display an error to the user or halt script execution
        // alert("UI Error: Could not initialize buttons.");
        // return;
    }























    // --- 2. Helper Functions ---

    /**
     * Shows a specific tab and hides others. Updates the visual indicator.
     * @param {string} tabId - The ID of the tab content element to show.
     * @param {number} index - The index (0-based) of the clicked tab button.
     */
    function showTab(tabId, index) {
      // Hide all tab content sections
      tabContents.forEach(tab => tab.classList.remove('active'));
      // Show the selected tab content section
      document.getElementById(tabId).classList.add('active');

      // Move the visual indicator under the active tab button
      tabIndicator.style.transform = `translateX(${index * 100}%)`;

      // Update the active state styling for the tab buttons
      tabButtons.forEach((button, i) => {
        button.classList.toggle('active', i === index); // Add 'active' class if i === index, remove otherwise
      });
    }

    /**
     * Handles the file(s) selected either by input or drag-and-drop.
     * Reads the content of the first file as text and displays it.
     * Lists the names of all selected files.
     * @param {FileList} files - The list of files from the input or drop event.
     */
    function handleFiles(files) {
      if (!files || files.length === 0) return; // Exit if no files

      fileList.innerHTML = ''; // Clear the previous list of file names

      // --- Process the first file for content display ---
      const file = files[0]; // We only read the content of the first file
      const reader = new FileReader();

      // When the file is successfully read
      reader.onload = () => {
        fileOutput.innerText = reader.result; // Display the file's text content
        // *** Potential Place for FETCH (File Tab) ***
        // If you want to send the file content to a server after reading:
        // sendTextToServer(reader.result, 'file');
      };

      // If there's an error reading the file
      reader.onerror = () => {
        console.error("Error reading file:", file.name);
        fileOutput.innerText = `Error reading file: ${file.name}`;
      };

      reader.readAsText(file); // Start reading the file as plain text

      // --- List all selected file names ---
      Array.from(files).forEach(f => {
        const li = document.createElement('li');
        li.textContent = f.name; // Display the name
        fileList.appendChild(li);
      });
    }

    /**
     * Shows a specific tab and hides others. Updates the visual indicator.
     * @param {string} tabId - The ID of the tab content element to show.
     * @param {number} index - The index (0-based) of the clicked tab button.
     */
    function showTab(tabId, index) {
      // Hide all tab content sections
      tabContents.forEach(tab => tab.classList.remove('active'));
      // Show the selected tab content section
      document.getElementById(tabId).classList.add('active');

      // Move the visual indicator under the active tab button
      tabIndicator.style.transform = `translateX(${index * 100}%)`;

      // Update the active state styling for the tab buttons
      tabButtons.forEach((button, i) => {
        button.classList.toggle('active', i === index); // Add 'active' class if i === index, remove otherwise
      });
    }

    /**
     * Handles the file(s) selected either by input or drag-and-drop.
     * Reads the content of the first file as text and displays it.
     * Lists the names of all selected files.
     * @param {FileList} files - The list of files from the input or drop event.
     */
    function handleFiles(files) {
      if (!files || files.length === 0) return; // Exit if no files

      fileList.innerHTML = ''; // Clear the previous list of file names

      // --- Process the first file for content display ---
      const file = files[0]; // We only read the content of the first file
      const reader = new FileReader();

      // When the file is successfully read
      reader.onload = () => {
        fileOutput.innerText = reader.result; // Display the file's text content
      };

      // If there's an error reading the file
      reader.onerror = () => {
        console.error("Error reading file:", file.name);
        fileOutput.innerText = `Error reading file: ${file.name}`;
      };

      reader.readAsText(file); // Start reading the file as plain text

      // --- List all selected file names ---
      Array.from(files).forEach(f => {
        const li = document.createElement('li');
        li.textContent = f.name; // Display the name
        fileList.appendChild(li);
      });
    }

    /**
     * Creates and triggers a download link for the given text content.
     * @param {string} text - The text content to download.
     * @param {string} filename - The desired name for the downloaded file.
     */
    function downloadTextFile(text, filename) {
      const blob = new Blob([text], { type: 'text/plain' }); // Create a blob from the text
      const link = document.createElement('a');             // Create a temporary link element
      link.href = URL.createObjectURL(blob);                // Set the link's target to the blob's URL
      link.download = filename;                             // Suggest the filename for download
      link.click();                                         // Programmatically click the link to trigger download
      URL.revokeObjectURL(link.href);                       // Clean up the blob URL
    }

     /**
     * Creates and triggers a download link for the given audio Blob.
     * @param {Blob} blob - The audio blob content to download.
     * @param {string} filename - The desired name for the downloaded file.
     */
    function downloadAudioFile(blob, filename) {
      const link = document.createElement('a');
      link.href = URL.createObjectURL(blob); // Create URL for the blob
      link.download = filename;              // Set the filename
      document.body.appendChild(link);       // Append link to body (needed for Firefox)
      link.click();                          // Programmatically click the link
      document.body.removeChild(link);       // Remove the link
      URL.revokeObjectURL(link.href);        // Clean up the blob URL
    }

    /**
     * Uses the browser's built-in speech synthesis to speak the provided text.
     * Note: This is kept for the file tab, but the text tab will use API audio.
     * @param {string} text - The text to be spoken.
     */
    function speakText(text) {
      if (!text || text.trim() === '') {
        console.warn("Attempted to speak empty text.");
        return;
      }
      // Stop any currently speaking utterance first to avoid overlap
      speechSynthesis.cancel();
      const utterance = new SpeechSynthesisUtterance(text);
      // You could add options here like utterance.voice, utterance.rate, etc.
      speechSynthesis.speak(utterance);
    }


    // --- 3. Tab Switching Logic ---
    // Add click listeners to each tab button
    tabButtons.forEach((button, index) => {
      button.addEventListener('click', () => {
        const tabId = button.dataset.tab; // Get the target tab ID from the button's data-tab attribute
        showTab(tabId, index);            // Show the corresponding tab
      });
    });

    // Show the first tab ('text') by default when the page loads
    showTab('text', 0);


    // --- State variable to hold the generated audio ---
    let generatedAudioBlob = null; // Will store the Blob from the API for the text tab
    let generatedAudioUrl = null;  // Will store the ObjectURL for the Blob

    // --- 4. "Text to Audio" Tab Event Listeners ---
    //-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    // Button 1: "Convertir" Text (Sends text to API, stores audio Blob)
    if (textProcessButton) { // Add null check before adding listener
      textProcessButton.addEventListener('click', async () => {
        const currentText = inputText.value.trim();
        outputText.innerText = currentText; // Display the input text

        // Reset previous audio state
        generatedAudioBlob = null;
        if (generatedAudioUrl) {
            URL.revokeObjectURL(generatedAudioUrl); // Clean up previous URL
            generatedAudioUrl = null;
        }
        // Optional: Disable play/download buttons until new audio is ready
        if(textSpeakButton) textSpeakButton.disabled = true; // Check if button exists
        if(textDownloadButton) textDownloadButton.disabled = true; // Check if button exists


        if (!currentText) {
          alert("Please enter some text to convert.");
          return;
        }

        textProcessButton.disabled = true;
        textProcessButton.textContent = 'Converting...';

        const apiUrl = 'http://127.0.0.1:5566/plustts';

        try {
          const response = await fetch(apiUrl, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({
              text: currentText
            })
          });

          if (!response.ok) {
            let errorMsg = `HTTP error! Status: ${response.status}`;
            try {
                const errorData = await response.text();
                errorMsg += `, Message: ${errorData}`;
            } catch (e) { /* Ignore */ }
            throw new Error(errorMsg);
          }

          const audioBlob = await response.blob();

          if (audioBlob.type.startsWith('audio/')) {
            generatedAudioBlob = audioBlob; // Store the received Blob
            generatedAudioUrl = URL.createObjectURL(generatedAudioBlob); // Create URL for playback/download

            console.log('Audio generated successfully. Ready for playback or download.');
            alert('Audio conversion successful!');

            // Enable play and download buttons now that audio is ready
            if(textSpeakButton) textSpeakButton.disabled = false;
            if(textDownloadButton) textDownloadButton.disabled = false;

          } else {
             console.warn('Received unexpected content type:', audioBlob.type);
             const responseText = await audioBlob.text();
             console.log("Server response (non-audio):", responseText);
             alert('Received unexpected response from server. Check console.');
             // Ensure buttons remain disabled if response wasn't audio
             if(textSpeakButton) textSpeakButton.disabled = true;
             if(textDownloadButton) textDownloadButton.disabled = true;
          }

        } catch (error) {
          console.error('Error fetching TTS audio:', error);
          alert(`Failed to generate audio: ${error.message}`);
           // Ensure buttons are re-enabled even on error, but kept disabled if no audio
          if(textSpeakButton) textSpeakButton.disabled = true;
          if(textDownloadButton) textDownloadButton.disabled = true;
        } finally {
          textProcessButton.disabled = false;
          textProcessButton.textContent = 'Convertir';
        }
      });
    } // End if (textProcessButton)

    // Button 2: "Reproducir" (Play the generated audio Blob)
    if (textSpeakButton) { // Add null check before adding listener
      textSpeakButton.addEventListener('click', () => {
        if (generatedAudioUrl) {
          const audio = new Audio(generatedAudioUrl); // Use the stored ObjectURL
          audio.play();
        } else {
          alert("Please convert the text to audio first using the 'Convertir' button.");
        }
      });
      // Disable initially until audio is generated
      textSpeakButton.disabled = true;
    } // End if (textSpeakButton)


    // Button 3: "Descargar" (Download the generated audio Blob)
    if (textDownloadButton) { // Add null check before adding listener
      textDownloadButton.addEventListener('click', () => {
        if (generatedAudioBlob) {
          // Try to determine a file extension, default to .wav
          let fileExtension = '.wav';
          if (generatedAudioBlob.type.includes('mpeg')) {
              fileExtension = '.mp3';
          } else if (generatedAudioBlob.type.includes('ogg')) {
              fileExtension = '.ogg';
          } // Add more types if needed
          downloadAudioFile(generatedAudioBlob, `generated-audio${fileExtension}`);
        } else {
          alert("Please convert the text to audio first using the 'Convertir' button.");
        }
      });
      // Disable initially until audio is generated
      textDownloadButton.disabled = true;
    } // End if (textDownloadButton)


    // --- 5. "File to Audio" Tab Event Listeners ---

    // --- 5a. Dropzone Setup ---
    // ... (keep dropzone listeners as they are) ...
    // Make the dropzone clickable to open the file selection dialog
    if (dropzone) {
      dropzone.addEventListener('click', (e) => {
        e.preventDefault(); // Prevent potential default actions
        if (fileInput) fileInput.click();  // Trigger the hidden file input
      });

        // Prevent the browser's default drag-and-drop behavior (opening the file)
      ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
        dropzone.addEventListener(eventName, (e) => {
          e.preventDefault();
          e.stopPropagation();
        }, false);
      });

      // Add visual feedback when dragging a file over the dropzone
      ['dragenter', 'dragover'].forEach(eventName => {
        dropzone.addEventListener(eventName, () => {
          dropzone.classList.add('dragover-highlight');
        }, false);
      });

      // Remove visual feedback when the dragged file leaves or is dropped
      ['dragleave', 'drop'].forEach(eventName => {
        dropzone.addEventListener(eventName, () => {
          dropzone.classList.remove('dragover-highlight');
        }, false);
      });

      // Handle files when they are dropped onto the dropzone
      dropzone.addEventListener('drop', (e) => {
        const files = e.dataTransfer.files; // Get the dropped files
        handleFiles(files);
      });
    } // End if (dropzone)

    // Handle files selected manually via the file input
    if (fileInput) {
      fileInput.addEventListener('change', () => {
        handleFiles(fileInput.files);
      });
    }























    // --- 5. "File to Audio" Tab Event Listeners ---

    // --- 5a. Dropzone Setup ---
    // Make the dropzone clickable to open the file selection dialog
    dropzone.addEventListener('click', (e) => {
      e.preventDefault(); // Prevent potential default actions
      fileInput.click();  // Trigger the hidden file input
    });

    // Handle files selected manually via the file input

    fileInput.addEventListener('change', () => {
      handleFiles(fileInput.files);
    });

    // Prevent the browser's default drag-and-drop behavior (opening the file)
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
      dropzone.addEventListener(eventName, (e) => {
        e.preventDefault();
        e.stopPropagation();
      }, false);
    });

    // Add visual feedback when dragging a file over the dropzone
    ['dragenter', 'dragover'].forEach(eventName => {
      dropzone.addEventListener(eventName, () => {
        dropzone.classList.add('dragover-highlight');
      }, false);
    });

    // Remove visual feedback when the dragged file leaves or is dropped
    ['dragleave', 'drop'].forEach(eventName => {
      dropzone.addEventListener(eventName, () => {
        dropzone.classList.remove('dragover-highlight');
      }, false);
    });

    // Handle files when they are dropped onto the dropzone
    dropzone.addEventListener('drop', (e) => {
      const files = e.dataTransfer.files; // Get the dropped files
      handleFiles(files);
    });

    // --- 5b. File Tab Button Actions ---
  //-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
    // Button 1: "Process" File (currently just shows an alert)
    fileProcessButton.addEventListener('click', () => {
      // *** Potential Place for FETCH (File Tab) ***
      // This is the most logical place if you want to send the *already read*
      // file content (from fileOutput) to your server for processing.
      const fileContent = fileOutput.innerText;
      if (fileContent) {
          // sendTextToServer(fileContent, 'file'); // Example call
          alert("File content ready for server processing (simulated).");
      } else {
          alert("No file content to process.");
      }
    });

    // Button 2: Speak File Content
    fileSpeakButton.addEventListener('click', () => {
      const textToSpeak = fileOutput.innerText; // Speak the text read from the file
      speakText(textToSpeak);
    }); 

    // Button 3: Download File Content (as .txt file)
    fileDownloadButton.addEventListener('click', () => {
      const textToDownload = fileOutput.innerText;
      downloadTextFile(textToDownload, 'file-content.txt');
    });



  }); // End of DOMContentLoaded listener