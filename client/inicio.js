document.getElementById("loginForm").addEventListener("submit", function(e) {
  e.preventDefault();

  const email = document.querySelector('input[type="email"]').value.trim();
  const password = document.querySelector('input[type="password"]').value.trim();
  const messageBox = document.getElementById("loginMessage");

  if (!email || !password) {
    showMessage("Por favor, completa todos los campos.", false);
    return;
  }

  // Aquí podrías agregar más validaciones si deseas
  // Por ahora, simulamos un login correcto
  showMessage("Inicio de sesión exitoso, redirigiendo...", true);

  // Redirigir después de un pequeño delay
  setTimeout(() => {
    window.location.href = "principal.html"; // Cambia esto según tu estructura
  }, 2000);
});

// Función para mostrar el mensaje animado
function showMessage(text, success = true) {
  let box = document.getElementById("loginMessage");

  if (!box) {
    box = document.createElement("div");
    box.id = "loginMessage";
    document.querySelector(".login-container").appendChild(box);
  }

  box.textContent = text;
  box.className = `login-message ${success ? "success" : "error"}`;

  // Animación de entrada
  box.classList.add("show");

  // Ocultar después de unos segundos
  setTimeout(() => {
    box.classList.remove("show");
  }, 3500);
}
