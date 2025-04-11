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
    button.addEventListener('click', () => {
      const tabId = button.dataset.tab;
      showTab(tabId, index);
    });
  });

  showTab('text', 0);

  // TEXTO A AUDIO
  document.querySelector('.tab-content#text .controls button:nth-child(1)').addEventListener('click', () => {
    const text = document.getElementById('inputText').value;
    document.getElementById('outputText').innerText = text;
  });

  document.querySelector('.tab-content#text .controls button:nth-child(2)').addEventListener('click', () => {
    const text = document.getElementById('outputText').innerText;
    const utterance = new SpeechSynthesisUtterance(text);
    speechSynthesis.speak(utterance);
  });

  document.querySelector('.tab-content#text .controls button:nth-child(3)').addEventListener('click', () => {
    const text = document.getElementById('outputText').innerText;
    const blob = new Blob([text], { type: 'text/plain' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = 'audio-simulacion.txt';
    link.click();
  });

  // ARCHIVO A AUDIO
  const dropzone = document.getElementById('dropzone');
  const fileInput = document.getElementById('file-input');
  const fileOutput = document.getElementById('fileOutput');
  const fileList = document.getElementById('file-list');

  // Bloquear comportamiento por defecto global de drag & drop
  window.addEventListener("dragover", e => e.preventDefault());
  window.addEventListener("drop", e => e.preventDefault());

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
    const files = e.dataTransfer.files;
    handleFiles(files);
  });

  fileInput.addEventListener('change', (e) => {
    const files = e.target.files;
    handleFiles(files);
  });

  function handleFiles(files) {
    if (!files || files.length === 0) return;

    fileList.innerHTML = ''; // limpiar lista
    const file = files[0]; // solo el primero para simular

    const reader = new FileReader();
    reader.onload = () => {
      fileOutput.innerText = reader.result;
    };
    reader.readAsText(file);

    Array.from(files).forEach(file => {
      const li = document.createElement('li');
      li.textContent = file.name;
      fileList.appendChild(li);
    });
  }

  document.querySelector('.tab-content#file .controls button:nth-child(1)').addEventListener('click', () => {
    alert("Archivo procesado y convertido (simulado).");
  });

  document.querySelector('.tab-content#file .controls button:nth-child(2)').addEventListener('click', () => {
    const text = fileOutput.innerText;
    const utterance = new SpeechSynthesisUtterance(text);
    speechSynthesis.speak(utterance);
  });

  document.querySelector('.tab-content#file .controls button:nth-child(3)').addEventListener('click', () => {
    const text = fileOutput.innerText;
    const blob = new Blob([text], { type: 'text/plain' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = 'archivo-convertido.txt';
    link.click();
  });
});
