// ui-functions.js
document.addEventListener("DOMContentLoaded", () => {
  // --- DOM Element References (UI Specific) ---
  const tabIndicator = document.getElementById('tabIndicator');
  const tabButtons = document.querySelectorAll('.tab-button');
  const tabContents = document.querySelectorAll('.tab-content');

  // --- Safety Check ---
  if (!tabIndicator || tabButtons.length === 0 || tabContents.length === 0) {
    console.error("Essential UI tab elements missing. Check HTML IDs and classes.");
    return; // Halt execution if critical elements are missing
  }

  // --- Helper Function: Tab Switching ---
  /**
   * Shows a specific tab and hides others. Updates the visual indicator.
   * @param {string} tabId - The ID of the tab content element to show.
   * @param {number} index - The index (0-based) of the clicked tab button.
   */
  function showTab(tabId, index) {
    // Hide all tab content sections
    tabContents.forEach(tab => tab?.classList.remove('active')); // Add null check

    // Show the selected tab content section
    const selectedTabContent = document.getElementById(tabId);
    if (selectedTabContent) {
      selectedTabContent.classList.add('active');
    } else {
      console.error(`Tab content with ID '${tabId}' not found.`);
      return; // Exit if the target tab content doesn't exist
    }

    // Move the visual indicator under the active tab button
    if (tabIndicator && tabButtons[index]) { // Check if elements exist
      const buttonWidth = tabButtons[index].offsetWidth || 100; // Default width
      tabIndicator.style.width = `${buttonWidth}px`;
      tabIndicator.style.transform = `translateX(${index * buttonWidth}px)`;
    }

    // Update the active state styling for the tab buttons
    tabButtons.forEach((button, i) => {
      button?.classList.toggle('active', i === index); // Add null check
    });
  }

  // --- Tab Switching Logic ---
  tabButtons.forEach((button, index) => {
    if (button) { // Check if button exists
      button.addEventListener('click', () => {
        const tabId = button.dataset.tab; // Get the target tab ID
        if (tabId) {
          showTab(tabId, index); // Show the corresponding tab
        } else {
          console.error("Tab button missing 'data-tab' attribute:", button);
        }
      });
    }
  });

  // --- Initial Tab Setup ---
  // Show the first tab ('text') by default when the page loads
  if (tabButtons.length > 0 && tabButtons[0].dataset.tab) {
    // Wait a tiny moment for layout calculation if needed for offsetWidth
    requestAnimationFrame(() => {
       showTab(tabButtons[0].dataset.tab, 0);
    });
  } else if (tabContents.length > 0 && tabContents[0].id) {
    // Fallback: Show the first content if buttons are misconfigured
    requestAnimationFrame(() => {
        showTab(tabContents[0].id, 0);
    });
  } else {
      console.warn("Could not determine the initial tab to display.");
  }

}); // End of DOMContentLoaded listener


function cerrarSesion() {
  localStorage.clear();
}
