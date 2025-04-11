document.getElementById("registerForm").addEventListener("submit", function (e) {
    e.preventDefault();
  
    const password = document.getElementById("password").value;
    const confirmPassword = document.getElementById("confirmPassword").value;
  
    if (password !== confirmPassword) {
      alert("Las contraseñas no coinciden. Por favor, intenta de nuevo.");
      return;
    }
  
    // Simulación de registro exitoso
    alert("Registro exitoso (simulado)");
  
    // Redirigir al login
    // window.location.href = "login.html";
  });
  