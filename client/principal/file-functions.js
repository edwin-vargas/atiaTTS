// file-functions.js - WebSocket Integration for Multi-File Upload and Conversion

document.addEventListener("DOMContentLoaded", () => {
  // --- WebSocket Configuration ---
  // IMPORTANT: Use a different endpoint for file processing if needed,
  // or ensure your server handles both text and file requests on the same endpoint.
  const WS_FILE_ENDPOINT = "/filetts"; // Example: distinct endpoint for files

  // --- DOM Element References (File Tab Specific) ---
  const fileTab = document.getElementById('file');
  const dropzone = document.getElementById('dropzone');
  const fileInput = document.getElementById('file-input');
  const fileListDisplay = document.getElementById('file-list'); // UL element for selected files
  const convertFilesBtn = document.getElementById('convert-files-btn');
  const fileConversionOutput = document.getElementById('fileConversionOutput'); // Div for results
  const statusEl = document.getElementById('status'); // Shared status display

  let socketFile = null; // WebSocket connection specifically for file operations
  let selectedFilesMap = new Map(); // Use Map for easier addition/removal: [fileId -> File object]
  let fileIdCounter = 0; // Simple counter for unique file IDs
  let receivedAudioCounter = 0; // Counter for naming downloaded files

  // --- Safety Check ---
  if (!fileTab || !dropzone || !fileInput || !fileListDisplay || !convertFilesBtn || !fileConversionOutput || !statusEl) {
      console.error("Essential file tab elements missing. Check HTML IDs: file, dropzone, file-input, file-list, convert-files-btn, fileConversionOutput, status");
      return;
  }

  // --- Helper Function: Shared Status Display ---
  function showStatus(message, isError = false) {
      if (!statusEl) return;
      // Simple approach: Append messages for file tab, might get long
      // Consider clearing status related to 'file' tab before showing a new one
      // statusEl.innerHTML = ''; // Uncomment to clear previous messages first

      const statusMsg = document.createElement('p'); // Display each as a paragraph
      statusMsg.textContent = `[Archivo Tab] ${message}`;
      statusMsg.className = isError ? 'error-msg' : 'success-msg'; // Simple class for potential styling
      // Apply basic styles directly or use CSS
      statusMsg.style.padding = '5px';
      statusMsg.style.margin = '2px 0';
      statusMsg.style.borderRadius = '3px';
      statusMsg.style.backgroundColor = isError ? '#ffebee' : '#e8f5e9';
      statusMsg.style.color = isError ? '#c62828' : '#2e7d32';

      statusEl.appendChild(statusMsg); // Append new status
      statusEl.style.display = 'block'; // Ensure main status container is visible

      // Auto-remove this specific message after a delay
      setTimeout(() => {
           if (statusMsg.parentNode === statusEl) { // Check if still attached
               statusEl.removeChild(statusMsg);
               // Hide main status container if empty
               if (!statusEl.hasChildNodes()) {
                   statusEl.style.display = 'none';
               }
           }
      }, 7000); // Longer timeout for potentially multiple messages
  }

  // --- WebSocket Connection Management (File Tab Specific) ---
  function connectFileWebSocket() {
      if (socketFile && (socketFile.readyState === WebSocket.OPEN || socketFile.readyState === WebSocket.CONNECTING)) {
          console.log("File WebSocket already open or connecting.");
          return;
      }

      const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const wsUrl = `${wsProtocol}//${window.location.hostname}:5566${WS_FILE_ENDPOINT}`; // Use specific endpoint
      console.log("Connecting to File WebSocket:", wsUrl);

      showStatus("Conectando al servidor de archivos...");
      convertFilesBtn.disabled = true;
      convertFilesBtn.textContent = "Conectando...";

      socketFile = new WebSocket(wsUrl);

      socketFile.onopen = () => {
          console.log("File WebSocket connection opened.");
          showStatus('Conexión para archivos establecida.');
          convertFilesBtn.disabled = false;
          convertFilesBtn.textContent = "Convertir Archivos";
      };

      socketFile.onmessage = (event) => {
          console.log("File WebSocket message received:", typeof event.data);
          if (event.data instanceof Blob) {
              // Assume server sends blob *after* sending metadata JSON
              // We need the metadata (like original filename) stored from the previous message
              handleReceivedAudioFile(event.data);
          } else {
              // Handle text/JSON messages (status, errors, metadata)
              try {
                  const messageData = JSON.parse(event.data);
                  console.log("Received JSON data:", messageData);

                  if (messageData.status === 'error') {
                      showStatus(`Error del servidor: ${messageData.message || 'Error desconocido'} (Archivo: ${messageData.originalFilename || 'N/A'})`, true);
                  } else if (messageData.status === 'success' && messageData.outputFilename && messageData.originalFilename) {
                       // Server confirms success *before* sending blob (good practice)
                       showStatus(`Conversión exitosa para: ${messageData.originalFilename}. Esperando audio...`);
                       // Store metadata needed when the blob arrives
                       // This assumes the blob *immediately* follows this message for the same file
                       socketFile.pendingFileInfo = messageData;
                  } else if (messageData.message) {
                      // General status message from server
                      showStatus(`Servidor: ${messageData.message}`);
                  } else {
                      showStatus(`Mensaje de texto inesperado: ${event.data}`);
                  }
              } catch (e) {
                  // Not JSON, treat as plain text status/error
                  showStatus(`Mensaje del servidor (texto): ${event.data}`);
              }
          }
      };

      socketFile.onclose = (event) => {
          console.log("File WebSocket connection closed:", event);
          const reason = event.reason ? ` Razón: ${event.reason}` : '';
          showStatus(`Conexión de archivos cerrada (Code: ${event.code})${reason}`, !event.wasClean);
          convertFilesBtn.disabled = true;
          convertFilesBtn.textContent = "Reconectando...";
          socketFile = null; // Clear the reference

          // Attempt to reconnect only if the file tab is active
          if (fileTab.classList.contains('active')) {
               setTimeout(connectFileWebSocket, 5000);
          }
      };

      socketFile.onerror = (error) => {
          console.error("File WebSocket Error:", error);
          showStatus('Error de conexión WebSocket para archivos. Verifica el servidor.', true);
          convertFilesBtn.disabled = true;
          convertFilesBtn.textContent = "Error de Conexión";
          socketFile = null; // Clear the reference
      };
  }

  // --- File Selection and Display ---

  /** Generates a unique ID for a file */
  function generateFileId() {
      return `file_${fileIdCounter++}`;
  }

  /** Updates the visual list of selected files (ul#file-list) */
  function updateSelectedFilesDisplay() {
      fileListDisplay.innerHTML = ''; // Clear current list
      if (selectedFilesMap.size === 0) {
           fileListDisplay.innerHTML = '<li>No hay archivos seleccionados.</li>'; // Placeholder
           return;
      }
      selectedFilesMap.forEach((file, fileId) => {
          const li = document.createElement('li');
          li.dataset.fileId = fileId; // Store ID for removal

          const fileNameSpan = document.createElement('span');
          fileNameSpan.textContent = `${file.name} (${(file.size / 1024).toFixed(1)} KB)`;
          li.appendChild(fileNameSpan);

          const removeBtn = document.createElement('button');
          removeBtn.textContent = 'X';
          removeBtn.className = 'remove-file-btn';
          removeBtn.title = 'Quitar archivo';
          removeBtn.onclick = () => removeFile(fileId);
          li.appendChild(removeBtn);

          fileListDisplay.appendChild(li);
      });
  }

   /** Adds files to the selection map and updates the display */
  function addFiles(files) {
      if (!files) return;
      let filesAdded = 0;
      for (const file of files) {
          // Optional: Add file type/size validation here
          const fileId = generateFileId();
          selectedFilesMap.set(fileId, file);
          filesAdded++;
      }
      if(filesAdded > 0) {
         updateSelectedFilesDisplay();
         showStatus(`${filesAdded} archivo(s) añadido(s) a la selección.`);
         // Clear placeholder in output area if needed
         clearOutputPlaceholder();
      }
  }

  /** Removes a file from the selection map and updates the display */
  function removeFile(fileId) {
      if (selectedFilesMap.has(fileId)) {
          const fileName = selectedFilesMap.get(fileId).name;
          selectedFilesMap.delete(fileId);
          updateSelectedFilesDisplay();
          showStatus(`Archivo quitado: ${fileName}`);
      }
  }

   /** Removes the initial placeholder text from the output area */
   function clearOutputPlaceholder() {
       const placeholder = fileConversionOutput.querySelector('p');
       if (placeholder && placeholder.textContent.includes('Los resultados')) {
           fileConversionOutput.removeChild(placeholder);
       }
   }

  // --- Drag and Drop Setup ---
  if (dropzone && fileInput) {
      dropzone.addEventListener('click', () => fileInput.click());

      ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
          dropzone.addEventListener(eventName, (e) => {
              e.preventDefault();
              e.stopPropagation();
          }, false);
      });
      ['dragenter', 'dragover'].forEach(eventName => dropzone.addEventListener(eventName, () => dropzone.classList.add('dragover-highlight')));
      ['dragleave', 'drop'].forEach(eventName => dropzone.addEventListener(eventName, () => dropzone.classList.remove('dragover-highlight')));

      dropzone.addEventListener('drop', (e) => {
          const files = e.dataTransfer?.files;
          if (files && files.length > 0) {
              addFiles(files); // Add dropped files
          }
      });

      fileInput.addEventListener('change', () => {
          if (fileInput.files && fileInput.files.length > 0) {
              addFiles(fileInput.files); // Add files from input
              fileInput.value = ''; // Reset input field to allow selecting the same file again
          }
      });
  }

  // --- File Conversion / Sending ---
  convertFilesBtn.addEventListener('click', () => {
      if (selectedFilesMap.size === 0) {
          showStatus("Por favor, selecciona uno o más archivos primero.", true);
          return;
      }

      if (!socketFile || socketFile.readyState !== WebSocket.OPEN) {
          showStatus("La conexión WebSocket para archivos no está activa. Intentando conectar...", true);
          connectFileWebSocket(); // Attempt connection
          return;
      }

      showStatus(`Iniciando conversión de ${selectedFilesMap.size} archivo(s)...`);
      convertFilesBtn.disabled = true; // Disable while sending
      convertFilesBtn.textContent = "Enviando...";
      clearOutputPlaceholder(); // Prepare output area

      // Send files one by one
      let filesSentCount = 0;
      selectedFilesMap.forEach((file, fileId) => {
          try {
               // 1. Send metadata first (optional but recommended)
               const metadata = {
                   type: 'file_upload_start', // Custom type to indicate file start
                   filename: file.name,
                   size: file.size,
                   fileId: fileId // Include the client-side ID if useful for server tracking
               };
               socketFile.send(JSON.stringify(metadata));
               console.log(`Sent metadata for ${file.name}`);

               // 2. Send the file blob
               socketFile.send(file); // Send the actual File object (WebSocket handles Blobs)
               console.log(`Sent blob for ${file.name}`);
               filesSentCount++;

          } catch (error) {
              console.error(`Error sending file ${file.name}:`, error);
              showStatus(`Error al enviar el archivo: ${file.name}`, true);
              // Decide whether to stop or continue with other files
          }
      });

      // Re-enable button after attempting to send all
      // Note: Actual completion is when server sends back results
      convertFilesBtn.disabled = false;
      convertFilesBtn.textContent = "Convertir Archivos";

      if (filesSentCount > 0) {
          showStatus(`${filesSentCount} archivo(s) enviados al servidor para procesar.`);
      }
      // Optionally clear the selection list after sending
      // selectedFilesMap.clear();
      // updateSelectedFilesDisplay();
  });

  // --- Handling Received Audio ---
  function handleReceivedAudioFile(audioBlob) {
       // Retrieve the metadata stored when the success message arrived
       const fileInfo = socketFile.pendingFileInfo;
       if (!fileInfo) {
           console.warn("Received an audio blob but no pending file info was found. Ignoring.");
            showStatus("Recibido audio inesperado sin información previa.", true);
           return;
       }
       // Clear pending info for the next file
       delete socketFile.pendingFileInfo;

      const originalFilename = fileInfo.originalFilename || 'Archivo Desconocido';
      const outputFilename = fileInfo.outputFilename || `audio_convertido_${receivedAudioCounter++}.wav`; // Default name

      const audioUrl = URL.createObjectURL(audioBlob);

      const resultItem = document.createElement('div');
      resultItem.className = 'file-item'; // Reuse existing style

      resultItem.innerHTML = `
          <div>
              <strong>Original:</strong> ${originalFilename}<br>
              <!-- <strong>Salida:</strong> ${outputFilename} (${(audioBlob.size / 1024).toFixed(1)} KB) -->
          </div>
          <div>
              <audio controls src="${audioUrl}" title="Reproducir ${outputFilename}"></audio>
              <a href="${audioUrl}" download="${outputFilename}" title="Descargar ${outputFilename}">Descargar</a>
          </div>
      `;

      fileConversionOutput.appendChild(resultItem); // Add to the results panel
      showStatus(`Audio recibido para: ${originalFilename}`);

      // Optional: Scroll the results panel to show the new item
      fileConversionOutput.scrollTop = fileConversionOutput.scrollHeight;

      // Note: Consider revoking audioUrl later to free memory, e.g., when the item is removed or page unloads.
  }


  // --- Tab Visibility Handling ---
  // Use MutationObserver to detect when the file tab becomes active/inactive
  // to connect/disconnect the WebSocket automatically.
  const observer = new MutationObserver((mutationsList) => {
      for (let mutation of mutationsList) {
          if (mutation.type === 'attributes' && mutation.attributeName === 'class') {
              const targetElement = mutation.target;
              if (targetElement.id === 'file') { // Observing the file tab content div
                  if (targetElement.classList.contains('active')) {
                      console.log("File tab activated - ensuring WebSocket connection.");
                      if (!socketFile || socketFile.readyState === WebSocket.CLOSED) {
                           connectFileWebSocket(); // Connect if not connected or closed
                      }
                  } else {
                      console.log("File tab deactivated - WebSocket connection left open for now.");
                      // Optional: Close the WebSocket if tab is inactive for a while
                      // if (socketFile && socketFile.readyState === WebSocket.OPEN) {
                      //     socketFile.close(1000, "Tab deactivated");
                      //     showStatus("Conexión de archivos cerrada por inactividad de pestaña.");
                      // }
                  }
              }
          }
      }
  });

  // Start observing the file tab content div for class changes
  if (fileTab) {
      observer.observe(fileTab, { attributes: true });
      // Initial check in case the file tab is active on page load
      if (fileTab.classList.contains('active')) {
           connectFileWebSocket();
      }
  } else {
      console.error("File tab content element (#file) not found for observer.");
  }

  // Initial setup
  updateSelectedFilesDisplay(); // Show initial placeholder in file list

}); // End of DOMContentLoaded listener