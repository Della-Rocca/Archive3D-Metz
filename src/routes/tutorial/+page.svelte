<script lang="ts">
  import { authStore } from "$lib/stores/auth";

  $: isAdmin = $authStore.role === "admin";

  type SectionId =
    | "intro"
    | "depot"
    | "archive"
    | "validation"
    | "settings"
    | "metadata";

  let activeSection: SectionId = "intro";

  const guestSections: { id: SectionId; label: string; icon: string }[] = [
    { id: "intro", label: "Introduction", icon: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" },
    { id: "depot", label: "Dépôt", icon: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" },
    { id: "archive", label: "Archive", icon: "M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8l1 12a2 2 0 002 2h8a2 2 0 002-2l1-12" },
  ];

  const adminSections: { id: SectionId; label: string; icon: string }[] = [
    { id: "validation", label: "Validation", icon: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" },
    { id: "settings", label: "Paramètres", icon: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z" },
    { id: "metadata", label: "Métadonnées", icon: "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" },
  ];

  $: allSections = isAdmin
    ? [...guestSections, ...adminSections]
    : guestSections;
</script>

<main class="tutorial-page">
  <div class="tutorial-layout">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <h2>Guide d'utilisation</h2>
        <span class="role-badge" class:admin={isAdmin}>
          {isAdmin ? "Admin" : "Invité"}
        </span>
      </div>
      <nav class="sidebar-nav">
        {#each allSections as section}
          <button
            class="nav-item"
            class:active={activeSection === section.id}
            class:admin-section={adminSections.some(s => s.id === section.id)}
            on:click={() => (activeSection = section.id)}
          >
            <svg viewBox="0 0 24 24" fill="none" width="18" height="18">
              <path d={section.icon} stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            {section.label}
          </button>
        {/each}
      </nav>
    </aside>

    <!-- Content -->
    <div class="content">

      <!-- ===== INTRODUCTION ===== -->
      {#if activeSection === "intro"}
        <section class="section" style="animation: fadeIn 0.2s ease">
          <div class="section-hero">
            <h1>Bienvenue sur Archive Metz</h1>
            <p class="hero-sub">
              Application de gestion et d'archivage de modèles 3D archéologiques.
              Ce guide vous explique comment utiliser toutes les fonctionnalités selon votre rôle.
            </p>
          </div>

          <div class="cards-grid">
            <div class="info-card">
              <div class="card-icon icon-blue">
                <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                  <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </div>
              <h3>Dépôt</h3>
              <p>Déposez vos structures archéologiques avec leurs fichiers 3D, photos et métadonnées en 4 étapes guidées.</p>
            </div>

            <div class="info-card">
              <div class="card-icon icon-amber">
                <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                  <path d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8l1 12a2 2 0 002 2h8a2 2 0 002-2l1-12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </div>
              <h3>Archive</h3>
              <p>Consultez les structures archivées, visualisez les modèles 3D et exportez les données d'inventaire.</p>
            </div>

            {#if isAdmin}
              <div class="info-card">
                <div class="card-icon icon-green">
                  <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                    <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </div>
                <h3>Validation</h3>
                <p>Examinez les dépôts en attente, corrigez les métadonnées et transférez les structures vers l'archive définitive.</p>
              </div>

              <div class="info-card">
                <div class="card-icon icon-slate">
                  <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                    <path d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065zM15 12a3 3 0 11-6 0 3 3 0 016 0z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </div>
                <h3>Paramètres</h3>
                <p>Configurez les chemins des dossiers de travail, gérez les listes de métadonnées et sécurisez l'accès admin.</p>
              </div>
            {/if}
          </div>

          <div class="workflow-block">
            <h2>Flux de travail</h2>
            <div class="workflow-steps">
              <div class="wf-step">
                <div class="wf-num">1</div>
                <div class="wf-body">
                  <strong>Dépôt</strong>
                  <span>Un utilisateur soumet une structure avec ses fichiers et métadonnées.</span>
                </div>
              </div>
              <div class="wf-arrow">→</div>
              <div class="wf-step">
                <div class="wf-num">2</div>
                <div class="wf-body">
                  <strong>Validation</strong>
                  <span>Un administrateur contrôle la qualité et corrige si besoin.</span>
                </div>
              </div>
              <div class="wf-arrow">→</div>
              <div class="wf-step">
                <div class="wf-num">3</div>
                <div class="wf-body">
                  <strong>Archivage</strong>
                  <span>La structure validée est transférée dans l'archive définitive.</span>
                </div>
              </div>
            </div>
          </div>
        </section>
      {/if}

      <!-- ===== DÉPÔT ===== -->
      {#if activeSection === "depot"}
        <section class="section" style="animation: fadeIn 0.2s ease">
          <div class="section-header">
            <div class="section-icon icon-blue">
              <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <div>
              <h1>Dépôt de structure</h1>
              <p>Soumettez une structure archéologique en 4 étapes guidées.</p>
            </div>
          </div>

          <div class="steps-list">
            <div class="step-item">
              <div class="step-badge">1</div>
              <div class="step-content">
                <h3>Opération — Rattachement</h3>
                <p>Associez la structure à une opération archéologique existante ou renseignez une nouvelle opération.</p>
                <div class="tip-box">
                  <strong>Champs obligatoires :</strong> Code opération, Site. Le bouton <em>Suivant</em> ne s'active qu'une fois ces champs remplis.
                </div>
                <ul class="field-list">
                  <li><strong>Code opération</strong> — Identifiant unique de l'opération (ex. : <code>OP-2024-001</code>). Tapez pour filtrer les opérations existantes.</li>
                  <li><strong>Site</strong> — Nom du site archéologique.</li>
                  <li><strong>Type d'opération</strong> — Catégorie (fouille préventive, programmée, etc.).</li>
                  <li><strong>Responsable</strong> — Nom du responsable de l'opération.</li>
                </ul>
              </div>
            </div>

            <div class="step-item">
              <div class="step-badge">2</div>
              <div class="step-content">
                <h3>Structure — Métadonnées</h3>
                <p>Décrivez la structure archéologique avec ses attributs techniques.</p>
                <div class="tip-box">
                  <strong>Champs obligatoires :</strong> Identifiant structure, Type de structure.
                </div>
                <ul class="field-list">
                  <li><strong>Identifiant structure</strong> — Code unique de la structure (ex. : <code>ST-001</code>).</li>
                  <li><strong>Type de structure</strong> — Catégorie typologique (fosse, fossé, mur, etc.).</li>
                  <li><strong>Description</strong> — Texte libre décrivant la structure.</li>
                  <li><strong>Auteur du modèle</strong> — Personne ayant réalisé la modélisation 3D.</li>
                  <li><strong>Déposant</strong> — Personne effectuant le dépôt.</li>
                  <li><strong>Nombre de photos</strong> — Rempli automatiquement selon les photos ajoutées à l'étape suivante.</li>
                  <li><strong>Nombre de faces</strong> — Calculé automatiquement pour les fichiers GLB/GLTF.</li>
                  <li><strong>Logiciels utilisés</strong> — Outils de photogrammétrie ou modélisation (Agisoft, Blender, etc.).</li>
                </ul>
              </div>
            </div>

            <div class="step-item">
              <div class="step-badge">3</div>
              <div class="step-content">
                <h3>Fichiers — Modèles & données</h3>
                <p>Ajoutez les fichiers associés à la structure. Vous pouvez glisser-déposer directement depuis votre explorateur de fichiers.</p>
                <div class="tip-box">
                  <strong>Obligatoire :</strong> Au moins un modèle 3D. Un seul modèle par structure est autorisé.
                </div>
                <div class="file-types-grid">
                  <div class="file-type">
                    <span class="ft-badge ft-model">3D</span>
                    <div>
                      <strong>Modèle 3D</strong>
                      <span class="ft-exts">.glb, .gltf, .obj</span>
                      <span class="ft-note">Un seul fichier. Le nombre de faces est calculé automatiquement pour GLB/GLTF.</span>
                    </div>
                  </div>
                  <div class="file-type">
                    <span class="ft-badge ft-ortho">Ortho</span>
                    <div>
                      <strong>Orthophotographies</strong>
                      <span class="ft-exts">.tif, .tiff, .png, .jpg, .jpeg</span>
                      <span class="ft-note">Orthomosaïques ou images de référence géoréférencées.</span>
                    </div>
                  </div>
                  <div class="file-type">
                    <span class="ft-badge ft-photo">Photo</span>
                    <div>
                      <strong>Photos</strong>
                      <span class="ft-exts">.jpg, .jpeg, .png, .tiff, .raw, .cr2, .nef, .arw, .dng</span>
                      <span class="ft-note">Le compteur de photos (étape 2) se met à jour automatiquement.</span>
                    </div>
                  </div>
                  <div class="file-type">
                    <span class="ft-badge ft-work">Work</span>
                    <div>
                      <strong>Fichiers de travail</strong>
                      <span class="ft-exts">Tous formats</span>
                      <span class="ft-note">Fichiers de projet, nuages de points, exports divers.</span>
                    </div>
                  </div>
                </div>
                <div class="tip-box tip-info" style="margin-top: var(--spacing-lg)">
                  <strong>Glisser-déposer :</strong> Faites glisser vos fichiers depuis l'explorateur directement sur la zone correspondante. Les doublons et les formats non autorisés sont automatiquement détectés et signalés.
                </div>
              </div>
            </div>

            <div class="step-item">
              <div class="step-badge">4</div>
              <div class="step-content">
                <h3>Récapitulatif — Vérification</h3>
                <p>Relisez toutes les informations avant de confirmer le dépôt. Vous pouvez revenir modifier n'importe quelle étape en cliquant sur <em>Modifier</em>.</p>
                <ul class="field-list">
                  <li>Les champs facultatifs non remplis sont signalés par un avertissement (en orange) — le dépôt reste possible.</li>
                  <li>Cliquez sur <strong>Déposer la structure</strong> pour finaliser.</li>
                  <li>En cas de succès, le formulaire se réinitialise automatiquement.</li>
                </ul>
              </div>
            </div>
          </div>

          {#if isAdmin}
            <div class="tip-box tip-admin">
              <strong>Administrateur :</strong> Le bouton <em>Gérer les métadonnées</em> (en haut à droite de la page Dépôt) vous permet de modifier les listes déroulantes (opérations, types, logiciels, etc.) sans quitter le formulaire.
            </div>
          {/if}
        </section>
      {/if}

      <!-- ===== ARCHIVE ===== -->
      {#if activeSection === "archive"}
        <section class="section" style="animation: fadeIn 0.2s ease">
          <div class="section-header">
            <div class="section-icon icon-amber">
              <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                <path d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8l1 12a2 2 0 002 2h8a2 2 0 002-2l1-12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <div>
              <h1>Archive</h1>
              <p>Consultez et explorez toutes les structures archivées.</p>
            </div>
          </div>

          <div class="feature-block">
            <h2>Recherche et filtres</h2>
            <ul class="field-list">
              <li><strong>Recherche libre</strong> — Filtrez par identifiant de structure, opération ou site.</li>
              <li><strong>Type de structure</strong> — Filtrez par catégorie typologique.</li>
              <li><strong>Opération</strong> — Limitez l'affichage à une opération spécifique.</li>
              <li><strong>Tags de révision</strong> — Affichez uniquement les structures taguées ou non taguées pour la révision.</li>
            </ul>
          </div>

          <div class="feature-block">
            <h2>Panneau de détail</h2>
            <p>Cliquez sur une structure dans la liste pour ouvrir son panneau de détail à droite. Vous y trouvez :</p>
            <ul class="field-list">
              <li><strong>Métadonnées complètes</strong> — Toutes les informations de l'opération et de la structure.</li>
              <li><strong>Visionneuse 3D</strong> — Visualisez le modèle directement dans l'application (rotation, zoom).</li>
              <li><strong>Fichiers associés</strong> — Liste des modèles, orthophotos, photos et fichiers de travail.</li>
              <li><strong>Informations techniques</strong> — Nombre de faces, taille des fichiers, date de dépôt.</li>
            </ul>
          </div>

          <div class="feature-block">
            <h2>Onglets du panneau</h2>
            <div class="cards-grid cards-grid-3">
              <div class="info-card">
                <h3>Structures</h3>
                <p>Liste paginée de toutes les structures archivées avec indicateurs de contenu (modèle 3D, orthophotos, fichiers de production).</p>
              </div>
              <div class="info-card">
                <h3>Statistiques</h3>
                <p>Vue globale de l'archive : total de structures, polygones, photos, répartition par type, opération, auteur et année.</p>
              </div>
              <div class="info-card">
                <h3>Inventaire</h3>
                <p>Tableau complet exportable avec toutes les métadonnées techniques par structure.</p>
              </div>
            </div>
          </div>

        </section>
      {/if}

      <!-- ===== VALIDATION (admin only) ===== -->
      {#if activeSection === "validation" && isAdmin}
        <section class="section" style="animation: fadeIn 0.2s ease">
          <div class="section-header">
            <div class="section-icon icon-green">
              <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <div>
              <h1>Validation</h1>
              <p>Contrôlez la qualité des dépôts avant de les archiver définitivement.</p>
            </div>
          </div>

          <div class="feature-block">
            <h2>Onglet "En attente"</h2>
            <p>Liste les structures qui ont été déposées mais pas encore archivées. Pour chaque structure :</p>
            <ul class="field-list">
              <li><strong>Sélectionnez une structure</strong> dans la liste de gauche pour ouvrir son panneau de détail.</li>
              <li><strong>Prévisualiser</strong> — Ouvre une fenêtre modale avec la visionneuse 3D du modèle.</li>
              <li><strong>Modifier les métadonnées</strong> — Corrigez ou complétez les champs directement depuis la validation.</li>
              <li><strong>Archiver</strong> — Transfère la structure vers l'archive définitive. Une confirmation est demandée.</li>
            </ul>
          </div>

          <div class="feature-block">
            <h2>Modification des métadonnées</h2>
            <p>En mode édition, vous pouvez modifier tous les champs de l'opération et de la structure. Les champs de logiciels supportent la sélection multiple via une liste déroulante.</p>
            <div class="tip-box">
              Cliquez sur <strong>Enregistrer</strong> pour sauvegarder les modifications avant d'archiver, ou sur <strong>Annuler</strong> pour revenir sans changer.
            </div>
          </div>

          <div class="feature-block">
            <h2>Tags de révision</h2>
            <p>Marquez une structure avec un tag de révision pour la retrouver facilement. Ajoutez une note optionnelle pour indiquer la raison du tag (ex. : "modèle à retraiter", "photos manquantes").</p>
            <ul class="field-list">
              <li>Les structures taguées apparaissent avec un indicateur dans la liste.</li>
              <li>Filtrez par tag depuis la page Archive pour isoler les structures à traiter.</li>
            </ul>
          </div>

          <div class="feature-block">
            <h2>Onglet "Historique"</h2>
            <p>Journal d'audit complet de toutes les actions effectuées : dépôts, validations, archivages, modifications. Chaque entrée affiche :</p>
            <ul class="field-list">
              <li>Date et heure de l'action</li>
              <li>Type d'action (dépôt, archivage, modification, suppression)</li>
              <li>Utilisateur ayant effectué l'action</li>
              <li>Chemin de la structure concernée</li>
              <li>Statut (succès ou erreur avec détail)</li>
            </ul>
            <div class="tip-box tip-warning">
              <strong>Attention :</strong> Le bouton <em>Réinitialiser l'historique</em> efface définitivement le journal d'audit. Cette action est irréversible.
            </div>
          </div>
        </section>
      {/if}

      <!-- ===== PARAMÈTRES (admin only) ===== -->
      {#if activeSection === "settings" && isAdmin}
        <section class="section" style="animation: fadeIn 0.2s ease">
          <div class="section-header">
            <div class="section-icon icon-slate">
              <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                <path d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065zM15 12a3 3 0 11-6 0 3 3 0 016 0z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <div>
              <h1>Paramètres</h1>
              <p>Configurez les chemins de travail et la sécurité de l'application.</p>
            </div>
          </div>

          <div class="feature-block">
            <h2>Chemins des dossiers</h2>
            <p>Chaque champ peut être modifié en cliquant sur l'icône de dossier à droite du champ pour ouvrir le sélecteur de fichier système.</p>
            <div class="paths-table">
              <div class="path-row">
                <div class="path-label">Dossier Dépôt</div>
                <div class="path-desc">Répertoire où les structures déposées sont stockées avant validation. Les utilisateurs qui font un dépôt y écrivent leurs fichiers.</div>
              </div>
              <div class="path-row">
                <div class="path-label">Dossier Validation</div>
                <div class="path-desc">Zone intermédiaire de contrôle qualité. Les structures y sont déplacées pendant la révision par l'administrateur.</div>
              </div>
              <div class="path-row">
                <div class="path-label">Dossier Archive</div>
                <div class="path-desc">Stockage définitif des structures validées et archivées. Toutes les structures visibles dans l'onglet Archive proviennent de ce répertoire.</div>
              </div>
              <div class="path-row">
                <div class="path-label">Fichier metadata-presets.json</div>
                <div class="path-desc">Référentiel JSON contenant les listes de valeurs prédéfinies (opérations, types, logiciels, auteurs, etc.) utilisées dans les formulaires.</div>
              </div>
              <div class="path-row">
                <div class="path-label">Dossier Logs</div>
                <div class="path-desc">Emplacement du journal d'audit. Le fichier <code>audit.log</code> y est écrit automatiquement par l'application.</div>
              </div>
            </div>
          </div>

          <div class="feature-block">
            <h2>Validation des chemins</h2>
            <p>Après modification d'un chemin, l'application vérifie automatiquement :</p>
            <ul class="field-list">
              <li>Que le chemin existe sur le système de fichiers.</li>
              <li>Que les permissions de lecture/écriture sont correctes.</li>
              <li>Que la structure de dossiers attendue est respectée.</li>
            </ul>
            <div class="tip-box">
              Les erreurs s'affichent en rouge sous le champ concerné. Les avertissements en orange signalent des situations non bloquantes (dossier vide, permissions limitées en lecture seule, etc.).
            </div>
          </div>

          <div class="feature-block">
            <h2>Mot de passe administrateur</h2>
            <p>Modifiez le mot de passe de connexion administrateur. La confirmation du nouveau mot de passe est requise pour éviter les erreurs de saisie.</p>
            <div class="tip-box tip-warning">
              <strong>Important :</strong> Notez votre nouveau mot de passe dans un endroit sûr. Il n'existe pas de mécanisme de récupération automatique.
            </div>
          </div>

          <div class="feature-block">
            <h2>Enregistrement</h2>
            <p>Cliquez sur <strong>Enregistrer la configuration</strong> pour appliquer tous les changements. La configuration est persistée localement dans un fichier de config de l'application.</p>
          </div>
        </section>
      {/if}

      <!-- ===== MÉTADONNÉES (admin only) ===== -->
      {#if activeSection === "metadata" && isAdmin}
        <section class="section" style="animation: fadeIn 0.2s ease">
          <div class="section-header">
            <div class="section-icon icon-purple">
              <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <div>
              <h1>Gestion des métadonnées</h1>
              <p>Gérez les listes de valeurs prédéfinies utilisées dans les formulaires de dépôt.</p>
            </div>
          </div>

          <div class="feature-block">
            <h2>Accès</h2>
            <p>L'éditeur de métadonnées est accessible depuis la page <strong>Dépôt</strong> via le bouton <em>Gérer les métadonnées</em> (en haut à droite). Il remplace temporairement le formulaire de dépôt.</p>
          </div>

          <div class="feature-block">
            <h2>Listes modifiables</h2>
            <div class="metadata-grid">
              <div class="meta-item">
                <strong>Opérations</strong>
                <span>Codes d'opérations archéologiques avec leurs sites associés. Chaque opération renseigne automatiquement le site lors de la sélection dans le formulaire.</span>
              </div>
              <div class="meta-item">
                <strong>Types de structure</strong>
                <span>Catégories typologiques des structures (fosse, fossé, silo, mur, etc.).</span>
              </div>
              <div class="meta-item">
                <strong>Types d'opération</strong>
                <span>Catégories d'opérations archéologiques (fouille préventive, sondage, prospection, etc.).</span>
              </div>
              <div class="meta-item">
                <strong>Logiciels</strong>
                <span>Outils de photogrammétrie et modélisation (Agisoft Metashape, Blender, CloudCompare, etc.).</span>
              </div>
              <div class="meta-item">
                <strong>Sites</strong>
                <span>Liste des sites archéologiques connus.</span>
              </div>
              <div class="meta-item">
                <strong>Responsables</strong>
                <span>Noms des responsables d'opération.</span>
              </div>
              <div class="meta-item">
                <strong>Auteurs de modèles</strong>
                <span>Noms des personnes réalisant les modélisations 3D.</span>
              </div>
              <div class="meta-item">
                <strong>Déposants</strong>
                <span>Noms des personnes effectuant les dépôts.</span>
              </div>
            </div>
          </div>

          <div class="feature-block">
            <h2>Modifier une liste</h2>
            <p>Pour chaque liste, vous pouvez :</p>
            <ul class="field-list">
              <li><strong>Ajouter</strong> une nouvelle valeur via le champ de saisie en bas de la liste.</li>
              <li><strong>Supprimer</strong> une valeur existante en cliquant sur l'icône × à côté de chaque entrée.</li>
            </ul>
            <div class="tip-box">
              Les modifications sont enregistrées dans le fichier <code>metadata-presets.json</code> défini dans les Paramètres. Après enregistrement, les nouvelles valeurs sont immédiatement disponibles dans les formulaires de dépôt.
            </div>
          </div>
        </section>
      {/if}

    </div>
  </div>
</main>

<style>
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .tutorial-page {
    min-height: calc(100vh - var(--app-topnav-height));
    background: var(--color-neutral-200);
  }

  .tutorial-layout {
    display: grid;
    grid-template-columns: 220px 1fr;
    min-height: calc(100vh - var(--app-topnav-height));
    max-width: 1200px;
    margin: 0 auto;
    gap: 0;
  }

  /* ===== SIDEBAR ===== */
  .sidebar {
    background: var(--color-neutral-100);
    border-right: 1px solid var(--color-neutral-400);
    padding: var(--spacing-xl) 0;
    position: sticky;
    top: var(--app-topnav-height);
    height: calc(100vh - var(--app-topnav-height));
    overflow-y: auto;
  }

  .sidebar-header {
    padding: 0 var(--spacing-lg) var(--spacing-lg);
    border-bottom: 1px solid var(--color-neutral-300);
    margin-bottom: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .sidebar-header h2 {
    margin: 0;
    font-size: var(--font-size-sm);
    font-weight: 700;
    color: var(--color-neutral-800);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .role-badge {
    display: inline-flex;
    align-items: center;
    padding: 2px var(--spacing-sm);
    border-radius: 999px;
    font-size: var(--font-size-xs);
    font-weight: 600;
    background: var(--color-neutral-300);
    color: var(--color-neutral-800);
    width: fit-content;
  }

  .role-badge.admin {
    background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
    color: #92400e;
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 var(--spacing-sm);
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--border-radius-md);
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--color-neutral-800);
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
    font-family: var(--font-family);
  }

  .nav-item:hover:not(.active) {
    background: var(--color-neutral-300);
  }

  .nav-item.active {
    background: var(--color-primary);
    color: var(--color-neutral-100);
  }

  .nav-item.admin-section {
    color: var(--color-neutral-700);
  }

  .nav-item.admin-section.active {
    color: var(--color-neutral-100);
  }

  /* ===== CONTENT ===== */
  .content {
    padding: var(--spacing-2xl);
    overflow-y: auto;
  }

  .section {
    max-width: 760px;
  }

  /* ===== SECTION HEADER ===== */
  .section-header {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-2xl);
    padding-bottom: var(--spacing-xl);
    border-bottom: 1px solid var(--color-neutral-300);
  }

  .section-header h1 {
    margin: 0 0 4px;
    font-size: var(--font-size-xl);
    color: var(--color-neutral-900);
    font-weight: 700;
  }

  .section-header p {
    margin: 0;
    color: var(--color-neutral-700);
    font-size: var(--font-size-sm);
  }

  .section-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    border-radius: var(--border-radius-lg);
    flex-shrink: 0;
  }

  /* ===== HERO ===== */
  .section-hero {
    margin-bottom: var(--spacing-2xl);
    padding-bottom: var(--spacing-xl);
    border-bottom: 1px solid var(--color-neutral-300);
  }

  .section-hero h1 {
    margin: 0 0 var(--spacing-md);
    font-size: var(--font-size-xl);
    color: var(--color-neutral-900);
    font-weight: 700;
  }

  .hero-sub {
    margin: 0;
    font-size: var(--font-size-base);
    color: var(--color-neutral-700);
    line-height: 1.7;
    max-width: 60ch;
  }

  /* ===== CARDS ===== */
  .cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-2xl);
  }

  .cards-grid-3 {
    grid-template-columns: repeat(3, 1fr);
  }

  .info-card {
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-lg);
    padding: var(--spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .info-card h3 {
    margin: 0;
    font-size: var(--font-size-sm);
    font-weight: 700;
    color: var(--color-neutral-900);
  }

  .info-card p {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--color-neutral-700);
    line-height: 1.6;
  }

  .card-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: var(--border-radius-md);
    margin-bottom: var(--spacing-sm);
  }

  /* ===== WORKFLOW ===== */
  .workflow-block {
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-lg);
    padding: var(--spacing-xl);
  }

  .workflow-block h2 {
    margin: 0 0 var(--spacing-lg);
    font-size: var(--font-size-base);
    font-weight: 700;
    color: var(--color-neutral-900);
  }

  .workflow-steps {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    flex-wrap: wrap;
  }

  .wf-step {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-md);
    flex: 1;
    min-width: 160px;
  }

  .wf-num {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--color-primary);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--font-size-xs);
    font-weight: 700;
    flex-shrink: 0;
  }

  .wf-body {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .wf-body strong {
    font-size: var(--font-size-sm);
    color: var(--color-neutral-900);
  }

  .wf-body span {
    font-size: var(--font-size-xs);
    color: var(--color-neutral-700);
    line-height: 1.5;
  }

  .wf-arrow {
    font-size: 1.2rem;
    color: var(--color-neutral-500);
    flex-shrink: 0;
  }

  /* ===== STEPS LIST ===== */
  .steps-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xl);
    margin-bottom: var(--spacing-xl);
  }

  .step-item {
    display: flex;
    gap: var(--spacing-lg);
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-lg);
    padding: var(--spacing-xl);
  }

  .step-badge {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--color-primary);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--font-size-sm);
    font-weight: 700;
    flex-shrink: 0;
  }

  .step-content {
    flex: 1;
  }

  .step-content h3 {
    margin: 0 0 var(--spacing-sm);
    font-size: var(--font-size-base);
    font-weight: 700;
    color: var(--color-neutral-900);
  }

  .step-content > p {
    margin: 0 0 var(--spacing-md);
    font-size: var(--font-size-sm);
    color: var(--color-neutral-700);
    line-height: 1.6;
  }

  /* ===== TIP BOX ===== */
  .tip-box {
    background: var(--color-info-bg);
    border: 1px solid var(--color-info-border);
    border-radius: var(--border-radius-md);
    padding: var(--spacing-md) var(--spacing-lg);
    font-size: var(--font-size-sm);
    color: var(--color-neutral-800);
    line-height: 1.6;
    margin-bottom: var(--spacing-md);
  }

  .tip-box:last-child {
    margin-bottom: 0;
  }

  .tip-box strong {
    color: var(--color-info);
  }

  .tip-box.tip-warning {
    background: var(--color-warning-bg);
    border-color: var(--color-warning-border);
  }

  .tip-box.tip-warning strong {
    color: var(--color-warning);
  }

  .tip-box.tip-admin {
    background: linear-gradient(135deg, #fef3c7 0%, #fffbeb 100%);
    border-color: #fde68a;
    margin-top: var(--spacing-xl);
  }

  .tip-box.tip-admin strong {
    color: #92400e;
  }

  .tip-box.tip-info strong {
    color: var(--color-info);
  }

  /* ===== FIELD LIST ===== */
  .field-list {
    margin: var(--spacing-md) 0 0;
    padding-left: var(--spacing-xl);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    font-size: var(--font-size-sm);
    color: var(--color-neutral-800);
    line-height: 1.6;
  }

  .field-list code {
    background: var(--color-neutral-300);
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 0.8em;
    font-family: ui-monospace, monospace;
  }

  /* ===== FILE TYPES ===== */
  .file-types-grid {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    margin-top: var(--spacing-md);
  }

  .file-type {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--color-neutral-200);
    border-radius: var(--border-radius-md);
  }

  .ft-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border-radius: var(--border-radius-md);
    font-size: var(--font-size-xs);
    font-weight: 700;
    flex-shrink: 0;
    letter-spacing: 0.02em;
  }

  .ft-model { background: var(--color-success-bg); color: var(--color-success); }
  .ft-ortho { background: var(--color-info-bg); color: var(--color-info); }
  .ft-photo { background: var(--color-warning-bg); color: var(--color-warning); }
  .ft-work  { background: rgba(30,58,95,0.08); color: var(--color-primary); }

  .file-type > div {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .file-type strong {
    font-size: var(--font-size-sm);
    color: var(--color-neutral-900);
  }

  .ft-exts {
    font-size: var(--font-size-xs);
    color: var(--color-neutral-600);
    font-family: ui-monospace, monospace;
  }

  .ft-note {
    font-size: var(--font-size-xs);
    color: var(--color-neutral-700);
    line-height: 1.5;
  }

  /* ===== FEATURE BLOCK ===== */
  .feature-block {
    margin-bottom: var(--spacing-2xl);
  }

  .feature-block h2 {
    margin: 0 0 var(--spacing-md);
    font-size: var(--font-size-base);
    font-weight: 700;
    color: var(--color-neutral-900);
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid var(--color-neutral-300);
  }

  .feature-block > p {
    margin: 0 0 var(--spacing-md);
    font-size: var(--font-size-sm);
    color: var(--color-neutral-700);
    line-height: 1.7;
  }

  /* ===== PATHS TABLE ===== */
  .paths-table {
    display: flex;
    flex-direction: column;
    gap: 1px;
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-md);
    overflow: hidden;
    margin-top: var(--spacing-md);
  }

  .path-row {
    display: grid;
    grid-template-columns: 200px 1fr;
    background: var(--color-neutral-100);
  }

  .path-row:not(:last-child) {
    border-bottom: 1px solid var(--color-neutral-300);
  }

  .path-label {
    padding: var(--spacing-md) var(--spacing-lg);
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--color-neutral-900);
    background: var(--color-neutral-200);
    border-right: 1px solid var(--color-neutral-300);
  }

  .path-desc {
    padding: var(--spacing-md) var(--spacing-lg);
    font-size: var(--font-size-sm);
    color: var(--color-neutral-700);
    line-height: 1.6;
  }

  .path-desc code {
    background: var(--color-neutral-300);
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 0.8em;
    font-family: ui-monospace, monospace;
  }

  /* ===== METADATA GRID ===== */
  .metadata-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-md);
    margin-top: var(--spacing-md);
  }

  .meta-item {
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-md);
    padding: var(--spacing-md) var(--spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .meta-item strong {
    font-size: var(--font-size-sm);
    color: var(--color-neutral-900);
  }

  .meta-item span {
    font-size: var(--font-size-xs);
    color: var(--color-neutral-700);
    line-height: 1.5;
  }

  /* ===== ICON COLORS ===== */
  .icon-blue   { background: var(--color-info-bg); color: var(--color-info); }
  .icon-amber  { background: var(--color-warning-bg); color: var(--color-warning); }
  .icon-green  { background: var(--color-success-bg); color: var(--color-success); }
  .icon-slate  { background: rgba(30,58,95,0.08); color: var(--color-primary); }
  .icon-purple { background: #ede9fe; color: #7c3aed; }

  /* ===== RESPONSIVE ===== */
  @media (max-width: 860px) {
    .tutorial-layout {
      grid-template-columns: 1fr;
    }

    .sidebar {
      position: static;
      height: auto;
      border-right: none;
      border-bottom: 1px solid var(--color-neutral-400);
      padding: var(--spacing-lg) var(--spacing-md);
    }

    .sidebar-nav {
      flex-direction: row;
      flex-wrap: wrap;
    }

    .content {
      padding: var(--spacing-lg);
    }

    .cards-grid-3 {
      grid-template-columns: 1fr;
    }

    .paths-table .path-row {
      grid-template-columns: 1fr;
    }

    .path-label {
      border-right: none;
      border-bottom: 1px solid var(--color-neutral-300);
    }

    .metadata-grid {
      grid-template-columns: 1fr;
    }

    .workflow-steps {
      flex-direction: column;
    }

    .wf-arrow {
      transform: rotate(90deg);
    }
  }
</style>
