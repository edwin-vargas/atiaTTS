document.addEventListener("DOMContentLoaded", () => {
  const registroForm = document.getElementById("registroForm");

  if (registroForm) {
    registroForm.addEventListener("submit", async function (e) {
      e.preventDefault();

      const inputs = registroForm.querySelectorAll("input");
      const user_name = inputs[0].value.trim();
      const user_email = inputs[1].value.trim();
      const user_pass = inputs[2].value.trim();

      if (!user_name || !user_email || !user_pass) {
        alert("Por favor completa todos los campos.");
        return;
      }

      try {
        const response = await fetch("http://localhost:5566/user", {
          method: "POST",
          headers: {
            "Content-Type": "application/json"
          },
          body: JSON.stringify({
            user_name,
            user_email,
            user_pass,
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
