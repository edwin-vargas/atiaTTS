document.getElementById("loginForm").addEventListener("submit", function(e) {
  e.preventDefault();

  const user_email = document.querySelector('input[type="email"]').value.trim();
  const user_pass = document.querySelector('input[type="password"]').value.trim();

  if (!user_email || !user_pass) {
    showMessage("Por favor, completa todos los campos.", false);
    return;
  }

  // Enviar datos al backend con fetch
  fetch("http://localhost:5566/signin", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      user_email: user_email,
      user_pass: user_pass
    })
  })
  .then(response => {
    if (!response.ok) {
      throw new Error("Credenciales inválidas o error en el servidor.");
    }
    return response.json();
  })
  .then(data => {
    console.log("Respuesta del backend:", data);
    showMessage("Inicio de sesión exitoso, redirigiendo...", true);

    localStorage.setItem("user_id", data.user_id)

    setTimeout(() => {
      window.location.href = "../principal/principal.html";
    }, 2000);
  })
  .catch(error => {
    console.error("Error al iniciar sesión:", error);
    showMessage("Error al iniciar sesión. Verifica tu correo y contraseña.", false);
  });
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
  box.className = `login-message ${success ? "success" : "error"} show`;

  setTimeout(() => {
    box.classList.remove("show");
  }, 3500);
}
