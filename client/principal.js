
document.addEventListener("DOMContentLoaded", () => {
    const tabIndicator = document.getElementById('tabIndicator');
    const tabButtons = document.querySelectorAll('.tab-button');
    const tabContents = document.querySelectorAll('.tab-content');
  
    function showTab(tabId, index) {
      tabContents.forEach(tab => tab.classList.remove('active'));
      document.getElementById(tabId).classList.add('active');
  
      tabIndicator.style.transform = `translateX(${index * 100}%)`;
  
      tabButtons.forEach((button, i) => {
        button.classList.toggle('active', i === index);
      });
    }
  
    tabButtons.forEach((button, index) => {
      button.addEventListener('click', () => showTab(button.dataset.tab, index));
    });
  
    // Inicializar primera pestaña
    showTab('text', 0);
  
    // Conversión de texto
    document.getElementById('convertText').addEventListener('click', () => {
      const text = document.getElementById('inputText').value;
      document.getElementById('outputText').innerText = text;
    });
  
    document.getElementById('speakText').addEventListener('click', () => {
      const text = document.getElementById('outputText').innerText;
      const utterance = new SpeechSynthesisUtterance(text);
      speechSynthesis.speak(utterance);
    });
  
    document.getElementById('downloadText').addEventListener('click', () => {
      const text = document.getElementById('outputText').innerText;
      const blob = new Blob([text], { type: 'text/plain' });
      const link = document.createElement('a');
      link.href = URL.createObjectURL(blob);
      link.download = 'audio-simulacion.txt';
      link.click();
    });
  
    // Dropzone funcionalidad
    const dropzone = document.getElementById('dropzone');
    const fileInput = document.getElementById('fileInput');
    const fileOutput = document.getElementById('fileOutput');
  
    dropzone.addEventListener('click', () => fileInput.click());
  
    dropzone.addEventListener('dragover', (e) => {
      e.preventDefault();
      dropzone.classList.add('dragover');
    });
  
    dropzone.addEventListener('dragleave', () => {
      dropzone.classList.remove('dragover');
    });
  
    dropzone.addEventListener('drop', (e) => {
      e.preventDefault();
      dropzone.classList.remove('dragover');
      const file = e.dataTransfer.files[0];
      handleFile(file);
    });
  
    fileInput.addEventListener('change', (e) => {
      const file = e.target.files[0];
      handleFile(file);
    });
  
    function handleFile(file) {
      if (!file) return;
      const reader = new FileReader();
      reader.onload = () => {
        fileOutput.innerText = reader.result;
      };
      reader.readAsText(file);
    }
  
    document.getElementById('convertFile').addEventListener('click', () => {
      alert("Archivo procesado y convertido (simulado).");
    });
  
    document.getElementById('speakFile').addEventListener('click', () => {
      const text = fileOutput.innerText;
      const utterance = new SpeechSynthesisUtterance(text);
      speechSynthesis.speak(utterance);
    });
  
    document.getElementById('downloadFile').addEventListener('click', () => {
      const text = fileOutput.innerText;
      const blob = new Blob([text], { type: 'text/plain' });
      const link = document.createElement('a');
      link.href = URL.createObjectURL(blob);
      link.download = 'archivo-convertido.txt';
      link.click();
    });
  });
  