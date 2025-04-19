document.addEventListener("DOMContentLoaded", () => {
  const registroForm = document.getElementById("registroForm");

  if (registroForm) {
    registroForm.addEventListener("submit", async function (e) {
      e.preventDefault();

      const inputs = registroForm.querySelectorAll("input");
      const nombre = inputs[0].value.trim();
      const correo = inputs[1].value.trim();
      const contraseña = inputs[2].value.trim();

      if (!nombre || !correo || !contraseña) {
        alert("Por favor completa todos los campos.");
        return;
      }

      try {
        const response = await fetch("https://gmq17x09-5566.usw3.devtunnels.ms/user", {
          method: "POST",
          headers: {
            "Content-Type": "application/json"
          },
          body: JSON.stringify({
            nombre,
            correo,
            contraseña
          })
        });

        if (!response.ok) {
          const error = await response.json();
          alert(`Error al registrar: ${error.message || "Algo salió mal."}`);
          return;
        }

        // Éxito
        const successMessage = document.getElementById("successMessage");
        successMessage.style.display = "block";

        setTimeout(() => {
          successMessage.style.display = "none";
          window.location.href = "planes.html";
        }, 2000);

      } catch (error) {
        console.error("Error de red:", error);
        alert("No se pudo completar el registro. Intenta más tarde.");
      }
    });
  }
});
