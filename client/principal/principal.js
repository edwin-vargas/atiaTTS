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

  // ARCHIVO A AUDIO - Dropzone 100% funcional
  const dropzone = document.getElementById('dropzone');
  const fileInput = document.getElementById('file-input');
  const fileList = document.getElementById('file-list');
  const fileOutput = document.getElementById('fileOutput');

  // 1. Hacer clic en el dropzone abre el input de archivos
  dropzone.addEventListener('click', (e) => {
    e.preventDefault(); // Evita que se active algún comportamiento raro
    fileInput.click();
  });

  // 2. Al seleccionar un archivo manualmente
  fileInput.addEventListener('change', () => {
    handleFiles(fileInput.files);
  });

  // 3. Previene que el navegador abra el archivo al soltarlo
  ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
    dropzone.addEventListener(eventName, (e) => {
      e.preventDefault();
      e.stopPropagation();
    }, false);
  });

  // 4. Estilo visual cuando arrastras encima del dropzone
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

  // 5. Maneja los archivos soltados
  dropzone.addEventListener('drop', (e) => {
    const files = e.dataTransfer.files;
    handleFiles(files);
  });

  // 6. Mostrar archivos y leer contenido
  function handleFiles(files) {
    if (!files || files.length === 0) return;

    fileList.innerHTML = '';
    const file = files[0]; // Solo procesamos el primer archivo

    const reader = new FileReader();
    reader.onload = () => {
      fileOutput.innerText = reader.result; // Mostramos el contenido del archivo
    };
    reader.readAsText(file); // Leemos el archivo como texto

    Array.from(files).forEach(file => {
      const li = document.createElement('li');
      li.textContent = file.name;
      fileList.appendChild(li);
    });
  }

  // Acciones de los botones en el panel de archivo
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
