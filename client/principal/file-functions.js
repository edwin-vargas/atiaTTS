// file-functions.js - WebSocket Integration for Multi-File Upload and Conversion

document.addEventListener("DOMContentLoaded", () => {
    // --- WebSocket Configuration ---
    const WS_FILE_ENDPOINT = "/filetts"; // Match endpoint with pro_tts_files.html
  
    // --- DOM Element References (File Tab Specific) ---
    const fileTab = document.getElementById('file');
    const dropzone = document.getElementById('dropzone');
    const fileInput = document.getElementById('file-input');
    const fileListDisplay = document.getElementById('file-list');
    const convertFilesBtn = document.getElementById('convert-files-btn');
    const fileConversionOutput = document.getElementById('fileConversionOutput');
    const statusEl = document.getElementById('status');
  
    let socketFile = null;
    let selectedFilesMap = new Map();
    let fileIdCounter = 0;
    let receivedAudioCounter = 0;
  
    // --- Safety Check ---
    if (!fileTab || !dropzone || !fileInput || !fileListDisplay || !convertFilesBtn || !fileConversionOutput || !statusEl) {
        console.error("Essential file tab elements missing. Check HTML IDs.");
        return;
    }
  
    // --- Helper Function: Shared Status Display ---
    function showStatus(message, isError = false) {
        if (!statusEl) return;
        
        const statusMsg = document.createElement('p');
        statusMsg.textContent = `[Archivo Tab] ${message}`;
        statusMsg.className = isError ? 'error-msg' : 'success-msg';
        statusMsg.style.padding = '5px';
        statusMsg.style.margin = '2px 0';
        statusMsg.style.borderRadius = '3px';
        statusMsg.style.backgroundColor = isError ? '#ffebee' : '#e8f5e9';
        statusMsg.style.color = isError ? '#c62828' : '#2e7d32';
  
        statusEl.appendChild(statusMsg);
        statusEl.style.display = 'block';
  
        setTimeout(() => {
             if (statusMsg.parentNode === statusEl) {
                 statusEl.removeChild(statusMsg);
                 if (!statusEl.hasChildNodes()) {
                     statusEl.style.display = 'none';
                 }
             }
        }, 7000);
    }
  
    // --- WebSocket Connection Management ---
    function connectFileWebSocket() {
        if (socketFile && (socketFile.readyState === WebSocket.OPEN || socketFile.readyState === WebSocket.CONNECTING)) {
            console.log("File WebSocket already open or connecting.");
            return;
        }
  
        const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${wsProtocol}//${window.location.hostname}:5566${WS_FILE_ENDPOINT}`;
        console.log("Connecting to File WebSocket:", wsUrl);
  
        showStatus("Conectando al servidor de archivos...");
        convertFilesBtn.disabled = true;
        convertFilesBtn.textContent = "Conectando...";
  
        socketFile = new WebSocket(wsUrl);
        // Set binary type to match pro_tts_files.html
        socketFile.binaryType = 'arraybuffer';
  
        socketFile.onopen = () => {
            console.log("File WebSocket connection opened.");
            showStatus('Conexión para archivos establecida.');
            convertFilesBtn.disabled = false;
            convertFilesBtn.textContent = "Convertir Archivos";
        };
  
        socketFile.onmessage = (event) => {
            console.log("File WebSocket message received:", typeof event.data);
            
            if (typeof event.data !== 'string') {
                // Binary data (ArrayBuffer) - create audio as in pro_tts_files.html
                const blob = new Blob([event.data], { type: 'audio/wav' });
                const audioUrl = URL.createObjectURL(blob);
                receivedAudioCounter++;
                
                const outputFilename = `audio_convertido_${receivedAudioCounter}.wav`;
                const originalFilename = Array.from(selectedFilesMap.values())[0]?.name || 'Archivo Desconocido';
                
                const resultItem = document.createElement('div');
                resultItem.className = 'file-item';
  
                resultItem.innerHTML = `
                    <div>
                        <strong>Original:</strong> ${originalFilename}
                    </div>
                    <div>
                        <audio controls src="${audioUrl}" title="Reproducir ${outputFilename}"></audio>
                        <a href="${audioUrl}" download="${outputFilename}" title="Descargar ${outputFilename}">Descargar</a>
                    </div>
                `;
  
                fileConversionOutput.appendChild(resultItem);
                showStatus(`Audio recibido para conversión.`);
                fileConversionOutput.scrollTop = fileConversionOutput.scrollHeight;
            } else {
                // Text message from server
                showStatus(`Servidor: ${event.data}`);
            }
        };
  
        socketFile.onclose = (event) => {
            console.log("File WebSocket connection closed:", event);
            const reason = event.reason ? ` Razón: ${event.reason}` : '';
            showStatus(`Conexión de archivos cerrada (Code: ${event.code})${reason}`, !event.wasClean);
            convertFilesBtn.disabled = true;
            convertFilesBtn.textContent = "Reconectando...";
            socketFile = null;
  
            if (fileTab.classList.contains('active')) {
                 setTimeout(connectFileWebSocket, 5000);
            }
        };
  
        socketFile.onerror = (error) => {
            console.error("File WebSocket Error:", error);
            showStatus('Error de conexión WebSocket para archivos. Verifica el servidor.', true);
            convertFilesBtn.disabled = true;
            convertFilesBtn.textContent = "Error de Conexión";
            socketFile = null;
        };
    }
  
    // --- File Selection and Display ---
    function generateFileId() {
        return `file_${fileIdCounter++}`;
    }
  
    function updateSelectedFilesDisplay() {
        fileListDisplay.innerHTML = '';
        if (selectedFilesMap.size === 0) {
             fileListDisplay.innerHTML = '<li>No hay archivos seleccionados.</li>';
             return;
        }
        selectedFilesMap.forEach((file, fileId) => {
            const li = document.createElement('li');
            li.dataset.fileId = fileId;
  
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
  
    function addFiles(files) {
        if (!files) return;
        let filesAdded = 0;
        for (const file of files) {
            // Filter for acceptable file types
            const acceptableTypes = ['.txt', '.pdf', '.docx'];
            const fileExt = file.name.substring(file.name.lastIndexOf('.')).toLowerCase();
            if (acceptableTypes.includes(fileExt) || file.type === 'text/plain') {
                const fileId = generateFileId();
                selectedFilesMap.set(fileId, file);
                filesAdded++;
            } else {
                showStatus(`Archivo ignorado: ${file.name} - Tipo de archivo no soportado`, true);
            }
        }
        if(filesAdded > 0) {
           updateSelectedFilesDisplay();
           showStatus(`${filesAdded} archivo(s) añadido(s) a la selección.`);
           clearOutputPlaceholder();
        }
    }
  
    function removeFile(fileId) {
        if (selectedFilesMap.has(fileId)) {
            const fileName = selectedFilesMap.get(fileId).name;
            selectedFilesMap.delete(fileId);
            updateSelectedFilesDisplay();
            showStatus(`Archivo quitado: ${fileName}`);
        }
    }
  
    function clearOutputPlaceholder() {
        const placeholder = fileConversionOutput.querySelector('p');
        if (placeholder && placeholder.textContent.includes('resultados')) {
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
                addFiles(files);
            }
        });
  
        fileInput.addEventListener('change', () => {
            if (fileInput.files && fileInput.files.length > 0) {
                addFiles(fileInput.files);
                fileInput.value = '';
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
            connectFileWebSocket();
            return;
        }
  
        showStatus(`Iniciando conversión de ${selectedFilesMap.size} archivo(s)...`);
        convertFilesBtn.disabled = true;
        convertFilesBtn.textContent = "Enviando...";
        clearOutputPlaceholder();
  
        // Send files one by one using ArrayBuffer (match pro_tts_files.html)
        let filesSentCount = 0;
        selectedFilesMap.forEach((file, fileId) => {
            try {
                 const reader = new FileReader();
                 reader.onload = () => {
                     // Send file data as ArrayBuffer (binary)
                     socketFile.send(reader.result);
                     console.log(`Sent file ${file.name} as ArrayBuffer`);
                     filesSentCount++;
                     
                     if (filesSentCount === selectedFilesMap.size) {
                         showStatus(`${filesSentCount} archivo(s) enviados al servidor para procesar.`);
                         convertFilesBtn.disabled = false;
                         convertFilesBtn.textContent = "Convertir Archivos";
                     }
                 };
                 reader.onerror = (error) => {
                     console.error(`Error reading file ${file.name}:`, error);
                     showStatus(`Error al leer el archivo: ${file.name}`, true);
                 };
                 // Start reading the file
                 reader.readAsArrayBuffer(file);
            } catch (error) {
                console.error(`Error sending file ${file.name}:`, error);
                showStatus(`Error al enviar el archivo: ${file.name}`, true);
            }
        });
    });
  
    // --- Tab Visibility Handling ---
    const observer = new MutationObserver((mutationsList) => {
        for (let mutation of mutationsList) {
            if (mutation.type === 'attributes' && mutation.attributeName === 'class') {
                const targetElement = mutation.target;
                if (targetElement.id === 'file') {
                    if (targetElement.classList.contains('active')) {
                        console.log("File tab activated - ensuring WebSocket connection.");
                        if (!socketFile || socketFile.readyState === WebSocket.CLOSED) {
                             connectFileWebSocket();
                        }
                    }
                }
            }
        }
    });
  
    if (fileTab) {
        observer.observe(fileTab, { attributes: true });
        if (fileTab.classList.contains('active')) {
             connectFileWebSocket();
        }
    } else {
        console.error("File tab content element (#file) not found for observer.");
    }
  
    // Initial setup
    updateSelectedFilesDisplay();
  
  }); // End of DOMContentLoaded listener