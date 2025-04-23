// text-functions.js - WebSocket Integration for Text-to-Speech

document.addEventListener("DOMContentLoaded", () => {
  let socket;
  let fileCounter = 1;

  // --- DOM Element References ---
  const inputText = document.getElementById('inputText');
  const voiceSelect = document.getElementById('voice-select');
  const convertBtn = document.getElementById('convert-text-btn');
  const audioFilesList = document.getElementById('audio-files-list');
  const statusEl = document.getElementById('status');

  // --- Safety Check ---
  if (!inputText || !voiceSelect || !convertBtn || !audioFilesList || !statusEl) {
      console.error("Essential text tab elements missing for WebSocket functionality. Check HTML IDs: inputText, voice-select, convert-text-btn, audio-files-list, status");
      return; // Stop if critical elements are missing
  }

  // --- Helper Functions ---
  function showStatus(message, isError = false) {
      if (!statusEl) return;
      statusEl.textContent = message;
      statusEl.className = 'status'; // Reset classes
      statusEl.classList.add(isError ? 'error' : 'success');
      statusEl.style.display = 'block';

      // Auto-hide after 5 seconds
      setTimeout(() => {
          if (statusEl) {
              statusEl.style.display = 'none';
          }
      }, 5000);
  }

  function connectWebSocket() {
      // Determine WebSocket protocol (ws/wss)
      const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      // Construct WebSocket URL (adjust hostname/port if needed)
      // Assuming the WebSocket server runs on the same host, port 5566
      const wsUrl = `${wsProtocol}//${window.location.hostname}:5566/protts`;
      console.log("Connecting to WebSocket:", wsUrl);

      // Disable button during connection attempt
      convertBtn.disabled = true;
      convertBtn.textContent = "Conectando...";

      socket = new WebSocket(wsUrl);

      socket.onopen = function(e) {
          showStatus('Conexión WebSocket establecida');
          convertBtn.disabled = false;
          convertBtn.textContent = "Convertir";
          console.log("WebSocket connection opened");
      };

      socket.onmessage = function(event) {
          console.log("WebSocket message received:", typeof event.data);
          if (event.data instanceof Blob) {
              // Received an audio file Blob
              handleAudioFile(event.data);
          } else {
              // Received a text message (likely status or error from server)
              try {
                  const messageData = JSON.parse(event.data);
                  // Check if it's a structured error/status message
                  if (messageData.status && messageData.message) {
                      showStatus(`Servidor: ${messageData.message}`, messageData.status === 'error');
                  } else {
                       showStatus(`Mensaje del servidor: ${event.data}`, true); // Assume error if format unknown
                  }
              } catch(e) {
                  // If not JSON, display as plain text (likely an error string)
                  showStatus(`Error del servidor: ${event.data}`, true);
              }

          }
      };

      socket.onclose = function(event) {
          console.log("WebSocket connection closed:", event);
          if (event.wasClean) {
              showStatus(`Conexión cerrada limpiamente (Code: ${event.code})`);
          } else {
              // e.g., server process killed or network down
              showStatus('Conexión WebSocket interrumpida. Intentando reconectar...', true);
          }
          convertBtn.disabled = true; // Disable button on close
          convertBtn.textContent = "Reconectando...";

          // Attempt to reconnect after a delay (e.g., 5 seconds)
          setTimeout(connectWebSocket, 5000);
      };

      socket.onerror = function(error) {
          console.error("WebSocket Error:", error);
          showStatus('Error de conexión WebSocket. Verifica que el servidor esté corriendo.', true);
          convertBtn.disabled = true; // Keep disabled on error
          convertBtn.textContent = "Error de Conexión";
          // Consider adding reconnect logic here too, possibly with backoff
      };
  }

  function handleAudioFile(blob) {
      // Create a URL for the audio Blob
      const audioUrl = URL.createObjectURL(blob);

      // Get the original text for context (optional preview)
      const originalText = inputText.value;
      const textPreview = originalText.length > 50 ? originalText.substring(0, 50) + '...' : originalText;

      // Clear the initial "No files" message if it exists
      const placeholder = audioFilesList.querySelector('p');
      if (placeholder) {
          audioFilesList.removeChild(placeholder);
      }

      // Create the file item element
      const fileItem = document.createElement('div');
      fileItem.className = 'file-item';

      const fileName = `audio_${fileCounter}.wav`; // Or derive from server if possible

      fileItem.innerHTML = `
          <div>
              <strong>Audio ${fileCounter}:</strong> "${textPreview}"
          </div>
          <div>
              <audio controls src="${audioUrl}"></audio>
              <a href="${audioUrl}" download="${fileName}">Descargar</a>
          </div>
      `;

      // Prepend to the list (newest first)
      audioFilesList.prepend(fileItem);

      // Increment counter for the next file
      fileCounter++;

      showStatus('Audio recibido y listo para reproducir/descargar.');

      // Optional: Clean up the object URL when it's no longer needed
      // This is tricky as the user might want to replay/download later.
      // A better approach might be to revoke URLs when the element is removed or page unloads.
      // fileItem.addEventListener('unload', () => URL.revokeObjectURL(audioUrl)); // Example, needs testing
  }

  // --- Event Listener for Convert Button ---
  convertBtn.addEventListener('click', function() {
      if (!socket || socket.readyState !== WebSocket.OPEN) {
          showStatus('No hay conexión con el servidor WebSocket. Intentando reconectar...', true);
          // Optionally try to reconnect immediately
          // if (socket && (socket.readyState === WebSocket.CLOSED || socket.readyState === WebSocket.CLOSING)) {
          //    connectWebSocket();
          // }
          return;
      }

      const text = inputText.value.trim();
      const voice = voiceSelect.value;

      if (!text) {
          showStatus('Por favor, ingresa un texto para convertir.', true);
          inputText.focus();
          return;
      }

      // Prepare message for the server
      const message = JSON.stringify({
          text: text,
          voice: voice
          // Add any other parameters the server expects (e.g., user_id, settings)
          // userId: localStorage.getItem("user_id") // Example
      });

      // Send the message via WebSocket
      try {
           socket.send(message);
           showStatus('Solicitud enviada al servidor...');
           // Optionally disable button while processing
           // convertBtn.disabled = true;
           // convertBtn.textContent = "Procesando...";
           // Re-enable in onmessage or onerror
      } catch (error) {
          console.error("Error sending message via WebSocket:", error);
          showStatus("Error al enviar la solicitud.", true);
      }

  });

  // --- Initial Connection ---
  connectWebSocket();

}); // End of DOMContentLoaded listener