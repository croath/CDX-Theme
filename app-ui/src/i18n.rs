use crate::types::Locale;

#[derive(Clone, Copy)]
pub struct I18n {
  pub locale: Locale,
}

impl I18n {
  pub fn t(self, key: &str) -> &'static str {
    translate(self.locale, key)
  }
}

pub fn translate(locale: Locale, key: &str) -> &'static str {
  match (locale, key) {
    // Nav
    (Locale::EnUs, "nav.recommend") => "Recommend",
    (Locale::ZhHans, "nav.recommend") => "推荐",
    (Locale::ZhHant, "nav.recommend") => "推薦",
    (Locale::JaJp, "nav.recommend") => "おすすめ",
    (Locale::KoKr, "nav.recommend") => "추천",
    (Locale::DeDe, "nav.recommend") => "Empfohlen",
    (Locale::EsEs, "nav.recommend") => "Recomendados",

    (Locale::EnUs, "nav.library") => "Library",
    (Locale::ZhHans, "nav.library") => "主题库",
    (Locale::ZhHant, "nav.library") => "主題庫",
    (Locale::JaJp, "nav.library") => "ライブラリ",
    (Locale::KoKr, "nav.library") => "라이브러리",
    (Locale::DeDe, "nav.library") => "Bibliothek",
    (Locale::EsEs, "nav.library") => "Biblioteca",

    (Locale::EnUs, "nav.install") => "Install",
    (Locale::ZhHans, "nav.install") => "安装",
    (Locale::ZhHant, "nav.install") => "安裝",
    (Locale::JaJp, "nav.install") => "インストール",
    (Locale::KoKr, "nav.install") => "설치",
    (Locale::DeDe, "nav.install") => "Installieren",
    (Locale::EsEs, "nav.install") => "Instalar",

    (Locale::EnUs, "nav.restore") => "Restore",
    (Locale::ZhHans, "nav.restore") => "恢复",
    (Locale::ZhHant, "nav.restore") => "還原",
    (Locale::JaJp, "nav.restore") => "復元",
    (Locale::KoKr, "nav.restore") => "복원",
    (Locale::DeDe, "nav.restore") => "Wiederherstellen",
    (Locale::EsEs, "nav.restore") => "Restaurar",

    (Locale::EnUs, "nav.settings") => "Settings",
    (Locale::ZhHans, "nav.settings") => "设置",
    (Locale::ZhHant, "nav.settings") => "設定",
    (Locale::JaJp, "nav.settings") => "設定",
    (Locale::KoKr, "nav.settings") => "설정",
    (Locale::DeDe, "nav.settings") => "Einstellungen",
    (Locale::EsEs, "nav.settings") => "Ajustes",

    // Brand
    (
      Locale::EnUs
      | Locale::ZhHans
      | Locale::ZhHant
      | Locale::JaJp
      | Locale::KoKr
      | Locale::DeDe
      | Locale::EsEs,
      "app.name",
    ) => "CDXTheme",

    (Locale::EnUs, "app.tagline") => "Themes for Codex",
    (Locale::ZhHans, "app.tagline") => "Codex 主题工具",
    (Locale::ZhHant, "app.tagline") => "Codex 主題工具",
    (Locale::JaJp, "app.tagline") => "Codex テーマツール",
    (Locale::KoKr, "app.tagline") => "Codex 테마 도구",
    (Locale::DeDe, "app.tagline") => "Themes für Codex",
    (Locale::EsEs, "app.tagline") => "Temas para Codex",

    // Theme toggle
    (Locale::EnUs, "theme.light") => "Light",
    (Locale::ZhHans, "theme.light") => "浅色",
    (Locale::ZhHant, "theme.light") => "淺色",
    (Locale::JaJp, "theme.light") => "ライト",
    (Locale::KoKr, "theme.light") => "라이트",
    (Locale::DeDe, "theme.light") => "Hell",
    (Locale::EsEs, "theme.light") => "Claro",

    (Locale::EnUs, "theme.dark") => "Dark",
    (Locale::ZhHans, "theme.dark") => "深色",
    (Locale::ZhHant, "theme.dark") => "深色",
    (Locale::JaJp, "theme.dark") => "ダーク",
    (Locale::KoKr, "theme.dark") => "다크",
    (Locale::DeDe, "theme.dark") => "Dunkel",
    (Locale::EsEs, "theme.dark") => "Oscuro",

    (Locale::EnUs, "theme.appearance") => "Appearance",
    (Locale::ZhHans, "theme.appearance") => "外观",
    (Locale::ZhHant, "theme.appearance") => "外觀",
    (Locale::JaJp, "theme.appearance") => "外観",
    (Locale::KoKr, "theme.appearance") => "모양",
    (Locale::DeDe, "theme.appearance") => "Erscheinungsbild",
    (Locale::EsEs, "theme.appearance") => "Apariencia",

    // Recommend
    (Locale::EnUs, "recommend.title") => "Recommended Themes",
    (Locale::ZhHans, "recommend.title") => "推荐主题",
    (Locale::ZhHant, "recommend.title") => "推薦主題",
    (Locale::JaJp, "recommend.title") => "おすすめテーマ",
    (Locale::KoKr, "recommend.title") => "추천 테마",
    (Locale::DeDe, "recommend.title") => "Empfohlene Themes",
    (Locale::EsEs, "recommend.title") => "Temas recomendados",

    (Locale::EnUs, "recommend.subtitle") => {
      "Curated looks from the cloud — apply downloads into your library"
    }
    (Locale::ZhHans, "recommend.subtitle") => "云端精选主题，应用时自动下载到本地主题库",
    (Locale::ZhHant, "recommend.subtitle") => "雲端精選主題，套用時自動下載到本地主題庫",
    (Locale::JaJp, "recommend.subtitle") => "クラウド厳選テーマ。適用時にライブラリへダウンロード",
    (Locale::KoKr, "recommend.subtitle") => {
      "클라우드 큐레이션 테마 — 적용 시 라이브러리에 다운로드"
    }
    (Locale::DeDe, "recommend.subtitle") => {
      "Kuratierte Cloud-Themes — beim Anwenden in die Bibliothek laden"
    }
    (Locale::EsEs, "recommend.subtitle") => {
      "Temas curados en la nube — al aplicar se descargan a tu biblioteca"
    }

    (Locale::EnUs, "recommend.apply") => "Apply",
    (Locale::ZhHans, "recommend.apply") => "应用",
    (Locale::ZhHant, "recommend.apply") => "套用",
    (Locale::JaJp, "recommend.apply") => "適用",
    (Locale::KoKr, "recommend.apply") => "적용",
    (Locale::DeDe, "recommend.apply") => "Anwenden",
    (Locale::EsEs, "recommend.apply") => "Aplicar",

    (Locale::EnUs, "recommend.apply.success") => "Theme applied",
    (Locale::ZhHans, "recommend.apply.success") => "主题已应用",
    (Locale::ZhHant, "recommend.apply.success") => "主題已套用",
    (Locale::JaJp, "recommend.apply.success") => "テーマを適用しました",
    (Locale::KoKr, "recommend.apply.success") => "테마가 적용되었습니다",
    (Locale::DeDe, "recommend.apply.success") => "Theme angewendet",
    (Locale::EsEs, "recommend.apply.success") => "Tema aplicado",

    (Locale::EnUs, "recommend.applying") => "Applying…",
    (Locale::ZhHans, "recommend.applying") => "应用中…",
    (Locale::ZhHant, "recommend.applying") => "套用中…",
    (Locale::JaJp, "recommend.applying") => "適用中…",
    (Locale::KoKr, "recommend.applying") => "적용 중…",
    (Locale::DeDe, "recommend.applying") => "Wird angewendet…",
    (Locale::EsEs, "recommend.applying") => "Aplicando…",

    (Locale::EnUs, "recommend.download") => "Download",
    (Locale::ZhHans, "recommend.download") => "下载",
    (Locale::ZhHant, "recommend.download") => "下載",
    (Locale::JaJp, "recommend.download") => "ダウンロード",
    (Locale::KoKr, "recommend.download") => "다운로드",
    (Locale::DeDe, "recommend.download") => "Herunterladen",
    (Locale::EsEs, "recommend.download") => "Descargar",

    (Locale::EnUs, "recommend.downloading") => "Downloading…",
    (Locale::ZhHans, "recommend.downloading") => "下载中…",
    (Locale::ZhHant, "recommend.downloading") => "下載中…",
    (Locale::JaJp, "recommend.downloading") => "ダウンロード中…",
    (Locale::KoKr, "recommend.downloading") => "다운로드 중…",
    (Locale::DeDe, "recommend.downloading") => "Wird heruntergeladen…",
    (Locale::EsEs, "recommend.downloading") => "Descargando…",

    (Locale::EnUs, "recommend.download.success") => "Theme saved to library",
    (Locale::ZhHans, "recommend.download.success") => "主题已保存到主题库",
    (Locale::ZhHant, "recommend.download.success") => "主題已儲存到主題庫",
    (Locale::JaJp, "recommend.download.success") => "ライブラリに保存しました",
    (Locale::KoKr, "recommend.download.success") => "라이브러리에 저장됨",
    (Locale::DeDe, "recommend.download.success") => "Theme in Bibliothek gespeichert",
    (Locale::EsEs, "recommend.download.success") => "Tema guardado en la biblioteca",

    (Locale::EnUs, "recommend.download.error") => "Download failed",
    (Locale::ZhHans, "recommend.download.error") => "下载失败",
    (Locale::ZhHant, "recommend.download.error") => "下載失敗",
    (Locale::JaJp, "recommend.download.error") => "ダウンロードに失敗",
    (Locale::KoKr, "recommend.download.error") => "다운로드 실패",
    (Locale::DeDe, "recommend.download.error") => "Download fehlgeschlagen",
    (Locale::EsEs, "recommend.download.error") => "Error al descargar",

    (Locale::EnUs, "recommend.applied") => "Applied",
    (Locale::ZhHans, "recommend.applied") => "已应用",
    (Locale::ZhHant, "recommend.applied") => "已套用",
    (Locale::JaJp, "recommend.applied") => "適用済み",
    (Locale::KoKr, "recommend.applied") => "적용됨",
    (Locale::DeDe, "recommend.applied") => "Aktiv",
    (Locale::EsEs, "recommend.applied") => "Aplicado",

    (Locale::EnUs, "recommend.loading") => "Loading themes…",
    (Locale::ZhHans, "recommend.loading") => "正在加载主题…",
    (Locale::ZhHant, "recommend.loading") => "正在載入主題…",
    (Locale::JaJp, "recommend.loading") => "テーマを読み込み中…",
    (Locale::KoKr, "recommend.loading") => "테마 불러오는 중…",
    (Locale::DeDe, "recommend.loading") => "Themes werden geladen…",
    (Locale::EsEs, "recommend.loading") => "Cargando temas…",

    (Locale::EnUs, "recommend.refresh") => "Refresh",
    (Locale::ZhHans, "recommend.refresh") => "刷新",
    (Locale::ZhHant, "recommend.refresh") => "重新整理",
    (Locale::JaJp, "recommend.refresh") => "更新",
    (Locale::KoKr, "recommend.refresh") => "새로고침",
    (Locale::DeDe, "recommend.refresh") => "Aktualisieren",
    (Locale::EsEs, "recommend.refresh") => "Actualizar",

    (Locale::EnUs, "recommend.refreshing") => "Refreshing…",
    (Locale::ZhHans, "recommend.refreshing") => "刷新中…",
    (Locale::ZhHant, "recommend.refreshing") => "重新整理中…",
    (Locale::JaJp, "recommend.refreshing") => "更新中…",
    (Locale::KoKr, "recommend.refreshing") => "새로고침 중…",
    (Locale::DeDe, "recommend.refreshing") => "Wird aktualisiert…",
    (Locale::EsEs, "recommend.refreshing") => "Actualizando…",

    (Locale::EnUs, "recommend.empty") => "No themes yet",
    (Locale::ZhHans, "recommend.empty") => "暂无主题",
    (Locale::ZhHant, "recommend.empty") => "暫無主題",
    (Locale::JaJp, "recommend.empty") => "テーマがありません",
    (Locale::KoKr, "recommend.empty") => "테마가 없습니다",
    (Locale::DeDe, "recommend.empty") => "Noch keine Themes",
    (Locale::EsEs, "recommend.empty") => "Aún no hay temas",

    (Locale::EnUs, "recommend.error") => "Failed to load themes",
    (Locale::ZhHans, "recommend.error") => "加载主题失败",
    (Locale::ZhHant, "recommend.error") => "載入主題失敗",
    (Locale::JaJp, "recommend.error") => "テーマの読み込みに失敗",
    (Locale::KoKr, "recommend.error") => "테마를 불러오지 못했습니다",
    (Locale::DeDe, "recommend.error") => "Themes konnten nicht geladen werden",
    (Locale::EsEs, "recommend.error") => "No se pudieron cargar los temas",

    (Locale::EnUs, "recommend.tag.builtin") => "built-in",
    (Locale::ZhHans, "recommend.tag.builtin") => "内置",
    (Locale::ZhHant, "recommend.tag.builtin") => "內建",
    (Locale::JaJp, "recommend.tag.builtin") => "内蔵",
    (Locale::KoKr, "recommend.tag.builtin") => "내장",
    (Locale::DeDe, "recommend.tag.builtin") => "integriert",
    (Locale::EsEs, "recommend.tag.builtin") => "integrado",

    (Locale::EnUs, "recommend.tag.install") => "install",
    (Locale::ZhHans, "recommend.tag.install") => "安装",
    (Locale::ZhHant, "recommend.tag.install") => "安裝",
    (Locale::JaJp, "recommend.tag.install") => "インストール",
    (Locale::KoKr, "recommend.tag.install") => "설치됨",
    (Locale::DeDe, "recommend.tag.install") => "installiert",
    (Locale::EsEs, "recommend.tag.install") => "instalado",

    (Locale::EnUs, "recommend.tag.remote") => "online",
    (Locale::ZhHans, "recommend.tag.remote") => "在线",
    (Locale::ZhHant, "recommend.tag.remote") => "線上",
    (Locale::JaJp, "recommend.tag.remote") => "オンライン",
    (Locale::KoKr, "recommend.tag.remote") => "온라인",
    (Locale::DeDe, "recommend.tag.remote") => "online",
    (Locale::EsEs, "recommend.tag.remote") => "en línea",

    (Locale::EnUs, "recommend.tag.update") => "update",
    (Locale::ZhHans, "recommend.tag.update") => "可更新",
    (Locale::ZhHant, "recommend.tag.update") => "可更新",
    (Locale::JaJp, "recommend.tag.update") => "更新あり",
    (Locale::KoKr, "recommend.tag.update") => "업데이트",
    (Locale::DeDe, "recommend.tag.update") => "Update",
    (Locale::EsEs, "recommend.tag.update") => "actualizar",

    (Locale::EnUs, "recommend.update") => "Update",
    (Locale::ZhHans, "recommend.update") => "更新",
    (Locale::ZhHant, "recommend.update") => "更新",
    (Locale::JaJp, "recommend.update") => "更新",
    (Locale::KoKr, "recommend.update") => "업데이트",
    (Locale::DeDe, "recommend.update") => "Aktualisieren",
    (Locale::EsEs, "recommend.update") => "Actualizar",

    (Locale::EnUs, "recommend.update.hint") => "New version available",
    (Locale::ZhHans, "recommend.update.hint") => "有新版本可用",
    (Locale::ZhHant, "recommend.update.hint") => "有新版本可用",
    (Locale::JaJp, "recommend.update.hint") => "新しいバージョンがあります",
    (Locale::KoKr, "recommend.update.hint") => "새 버전 사용 가능",
    (Locale::DeDe, "recommend.update.hint") => "Neue Version verfügbar",
    (Locale::EsEs, "recommend.update.hint") => "Nueva versión disponible",

    (Locale::EnUs, "recommend.update.notify") => "Theme updates available",
    (Locale::ZhHans, "recommend.update.notify") => "有主题可更新",
    (Locale::ZhHant, "recommend.update.notify") => "有主題可更新",
    (Locale::JaJp, "recommend.update.notify") => "テーマの更新があります",
    (Locale::KoKr, "recommend.update.notify") => "업데이트 가능한 테마가 있습니다",
    (Locale::DeDe, "recommend.update.notify") => "Theme-Updates verfügbar",
    (Locale::EsEs, "recommend.update.notify") => "Hay actualizaciones de temas",

    (Locale::EnUs, "library.title") => "Theme Library",
    (Locale::ZhHans, "library.title") => "主题库",
    (Locale::ZhHant, "library.title") => "主題庫",
    (Locale::JaJp, "library.title") => "テーマライブラリ",
    (Locale::KoKr, "library.title") => "테마 라이브러리",
    (Locale::DeDe, "library.title") => "Theme-Bibliothek",
    (Locale::EsEs, "library.title") => "Biblioteca de temas",

    (Locale::EnUs, "library.subtitle") => "Built-in and downloaded packages on this device",
    (Locale::ZhHans, "library.subtitle") => "本机内置与已下载的主题包",
    (Locale::ZhHant, "library.subtitle") => "本機內建與已下載的主題包",
    (Locale::JaJp, "library.subtitle") => "この端末の内蔵・ダウンロード済みテーマ",
    (Locale::KoKr, "library.subtitle") => "이 기기의 내장 및 다운로드된 패키지",
    (Locale::DeDe, "library.subtitle") => {
      "Integrierte und heruntergeladene Pakete auf diesem Gerät"
    }
    (Locale::EsEs, "library.subtitle") => "Paquetes integrados y descargados en este dispositivo",

    (Locale::EnUs, "library.loading") => "Loading library…",
    (Locale::ZhHans, "library.loading") => "正在加载主题库…",
    (Locale::ZhHant, "library.loading") => "正在載入主題庫…",
    (Locale::JaJp, "library.loading") => "ライブラリを読み込み中…",
    (Locale::KoKr, "library.loading") => "라이브러리 불러오는 중…",
    (Locale::DeDe, "library.loading") => "Bibliothek wird geladen…",
    (Locale::EsEs, "library.loading") => "Cargando biblioteca…",

    (Locale::EnUs, "library.empty") => "No installed themes yet — apply one from Recommend",
    (Locale::ZhHans, "library.empty") => "暂无已安装主题，请从推荐页应用",
    (Locale::ZhHant, "library.empty") => "暫無已安裝主題，請從推薦頁套用",
    (Locale::JaJp, "library.empty") => "インストール済みテーマはありません（おすすめから適用）",
    (Locale::KoKr, "library.empty") => "설치된 테마가 없습니다 — 추천에서 적용하세요",
    (Locale::DeDe, "library.empty") => "Noch keine Themes — unter Empfohlen anwenden",
    (Locale::EsEs, "library.empty") => "Aún no hay temas instalados — aplica uno en Recomendados",

    (Locale::EnUs, "library.error") => "Failed to load library",
    (Locale::ZhHans, "library.error") => "加载主题库失败",
    (Locale::ZhHant, "library.error") => "載入主題庫失敗",
    (Locale::JaJp, "library.error") => "ライブラリの読み込みに失敗",
    (Locale::KoKr, "library.error") => "라이브러리를 불러오지 못했습니다",
    (Locale::DeDe, "library.error") => "Bibliothek konnte nicht geladen werden",
    (Locale::EsEs, "library.error") => "No se pudo cargar la biblioteca",

    (Locale::EnUs, "recommend.delete") => "Delete",
    (Locale::ZhHans, "recommend.delete") => "删除",
    (Locale::ZhHant, "recommend.delete") => "刪除",
    (Locale::JaJp, "recommend.delete") => "削除",
    (Locale::KoKr, "recommend.delete") => "삭제",
    (Locale::DeDe, "recommend.delete") => "Löschen",
    (Locale::EsEs, "recommend.delete") => "Eliminar",

    (Locale::EnUs, "recommend.deleting") => "Deleting…",
    (Locale::ZhHans, "recommend.deleting") => "删除中…",
    (Locale::ZhHant, "recommend.deleting") => "刪除中…",
    (Locale::JaJp, "recommend.deleting") => "削除中…",
    (Locale::KoKr, "recommend.deleting") => "삭제 중…",
    (Locale::DeDe, "recommend.deleting") => "Wird gelöscht…",
    (Locale::EsEs, "recommend.deleting") => "Eliminando…",

    (Locale::EnUs, "recommend.delete.success") => "Theme deleted",
    (Locale::ZhHans, "recommend.delete.success") => "主题已删除",
    (Locale::ZhHant, "recommend.delete.success") => "主題已刪除",
    (Locale::JaJp, "recommend.delete.success") => "テーマを削除しました",
    (Locale::KoKr, "recommend.delete.success") => "테마가 삭제되었습니다",
    (Locale::DeDe, "recommend.delete.success") => "Theme gelöscht",
    (Locale::EsEs, "recommend.delete.success") => "Tema eliminado",

    (Locale::EnUs, "recommend.delete.error") => "Delete failed",
    (Locale::ZhHans, "recommend.delete.error") => "删除失败",
    (Locale::ZhHant, "recommend.delete.error") => "刪除失敗",
    (Locale::JaJp, "recommend.delete.error") => "削除に失敗",
    (Locale::KoKr, "recommend.delete.error") => "삭제 실패",
    (Locale::DeDe, "recommend.delete.error") => "Löschen fehlgeschlagen",
    (Locale::EsEs, "recommend.delete.error") => "Error al eliminar",

    (Locale::EnUs, "recommend.delete.confirm.title") => "Delete theme?",
    (Locale::ZhHans, "recommend.delete.confirm.title") => "删除主题？",
    (Locale::ZhHant, "recommend.delete.confirm.title") => "刪除主題？",
    (Locale::JaJp, "recommend.delete.confirm.title") => "テーマを削除しますか？",
    (Locale::KoKr, "recommend.delete.confirm.title") => "테마를 삭제할까요?",
    (Locale::DeDe, "recommend.delete.confirm.title") => "Theme löschen?",
    (Locale::EsEs, "recommend.delete.confirm.title") => "¿Eliminar tema?",

    (Locale::EnUs, "recommend.delete.confirm.body") => {
      "This will remove the package from your library. This action cannot be undone."
    }
    (Locale::ZhHans, "recommend.delete.confirm.body") => {
      "将从本地库中移除该主题包，此操作无法撤销。"
    }
    (Locale::ZhHant, "recommend.delete.confirm.body") => {
      "將從本機庫中移除此主題包，此操作無法復原。"
    }
    (Locale::JaJp, "recommend.delete.confirm.body") => {
      "ライブラリからこのパッケージを削除します。この操作は元に戻せません。"
    }
    (Locale::KoKr, "recommend.delete.confirm.body") => {
      "라이브러리에서 이 패키지를 제거합니다. 이 작업은 취소할 수 없습니다."
    }
    (Locale::DeDe, "recommend.delete.confirm.body") => {
      "Dadurch wird das Paket aus Ihrer Bibliothek entfernt. Dies kann nicht rückgängig gemacht werden."
    }
    (Locale::EsEs, "recommend.delete.confirm.body") => {
      "Esto eliminará el paquete de tu biblioteca. Esta acción no se puede deshacer."
    }

    (Locale::EnUs, "recommend.delete.confirm.ok") => "Delete",
    (Locale::ZhHans, "recommend.delete.confirm.ok") => "删除",
    (Locale::ZhHant, "recommend.delete.confirm.ok") => "刪除",
    (Locale::JaJp, "recommend.delete.confirm.ok") => "削除",
    (Locale::KoKr, "recommend.delete.confirm.ok") => "삭제",
    (Locale::DeDe, "recommend.delete.confirm.ok") => "Löschen",
    (Locale::EsEs, "recommend.delete.confirm.ok") => "Eliminar",

    (Locale::EnUs, "recommend.delete.confirm.cancel") => "Cancel",
    (Locale::ZhHans, "recommend.delete.confirm.cancel") => "取消",
    (Locale::ZhHant, "recommend.delete.confirm.cancel") => "取消",
    (Locale::JaJp, "recommend.delete.confirm.cancel") => "キャンセル",
    (Locale::KoKr, "recommend.delete.confirm.cancel") => "취소",
    (Locale::DeDe, "recommend.delete.confirm.cancel") => "Abbrechen",
    (Locale::EsEs, "recommend.delete.confirm.cancel") => "Cancelar",

    // Install
    (Locale::EnUs, "install.title") => "Install Theme",
    (Locale::ZhHans, "install.title") => "安装主题",
    (Locale::ZhHant, "install.title") => "安裝主題",
    (Locale::JaJp, "install.title") => "テーマをインストール",
    (Locale::KoKr, "install.title") => "테마 설치",
    (Locale::DeDe, "install.title") => "Theme installieren",
    (Locale::EsEs, "install.title") => "Instalar tema",

    (Locale::EnUs, "install.subtitle") => "Install a CDXTheme package into your library",
    (Locale::ZhHans, "install.subtitle") => "安装 CDXTheme 主题包到本地库",
    (Locale::ZhHant, "install.subtitle") => "安裝 CDXTheme 主題包到本機庫",
    (Locale::JaJp, "install.subtitle") => "CDXTheme パッケージをライブラリに追加",
    (Locale::KoKr, "install.subtitle") => "CDXTheme 패키지를 라이브러리에 설치",
    (Locale::DeDe, "install.subtitle") => "CDXTheme-Paket in die Bibliothek installieren",
    (Locale::EsEs, "install.subtitle") => "Instala un paquete CDXTheme en tu biblioteca",

    (Locale::EnUs, "install.drop") => "Drop a .cdxtheme file here",
    (Locale::ZhHans, "install.drop") => "将 .cdxtheme 主题包拖放到此处",
    (Locale::ZhHant, "install.drop") => "將 .cdxtheme 主題包拖放到此處",
    (Locale::JaJp, "install.drop") => ".cdxtheme ファイルをここにドロップ",
    (Locale::KoKr, "install.drop") => ".cdxtheme 파일을 여기에 놓으세요",
    (Locale::DeDe, "install.drop") => ".cdxtheme-Datei hier ablegen",
    (Locale::EsEs, "install.drop") => "Suelta un archivo .cdxtheme aquí",

    (Locale::EnUs, "install.or") => "or",
    (Locale::ZhHans, "install.or") => "或",
    (Locale::ZhHant, "install.or") => "或",
    (Locale::JaJp, "install.or") => "または",
    (Locale::KoKr, "install.or") => "또는",
    (Locale::DeDe, "install.or") => "oder",
    (Locale::EsEs, "install.or") => "o",

    (Locale::EnUs, "install.browse") => "Choose file",
    (Locale::ZhHans, "install.browse") => "选择文件",
    (Locale::ZhHant, "install.browse") => "選擇檔案",
    (Locale::JaJp, "install.browse") => "ファイルを選択",
    (Locale::KoKr, "install.browse") => "파일 선택",
    (Locale::DeDe, "install.browse") => "Datei wählen",
    (Locale::EsEs, "install.browse") => "Elegir archivo",

    (Locale::EnUs, "install.hint") => "Supports multi-app packages (.cdxtheme · max 30MB).",
    (Locale::ZhHans, "install.hint") => "支持多应用主题包（.cdxtheme · 最大 30MB）。",
    (Locale::ZhHant, "install.hint") => "支援多應用主題包（.cdxtheme · 最大 30MB）。",
    (Locale::JaJp, "install.hint") => "マルチアプリパッケージ対応（.cdxtheme · 最大 30MB）。",
    (Locale::KoKr, "install.hint") => "멀티 앱 패키지 지원(.cdxtheme · 최대 30MB).",
    (Locale::DeDe, "install.hint") => "Unterstützt Multi-App-Pakete (.cdxtheme · max. 30 MB).",
    (Locale::EsEs, "install.hint") => "Admite paquetes multiapp (.cdxtheme · máx. 30 MB).",

    (Locale::EnUs, "install.installing") => "Installing…",
    (Locale::ZhHans, "install.installing") => "安装中…",
    (Locale::ZhHant, "install.installing") => "安裝中…",
    (Locale::JaJp, "install.installing") => "インストール中…",
    (Locale::KoKr, "install.installing") => "설치 중…",
    (Locale::DeDe, "install.installing") => "Wird installiert…",
    (Locale::EsEs, "install.installing") => "Instalando…",

    (Locale::EnUs, "install.success") => "Theme installed",
    (Locale::ZhHans, "install.success") => "主题已安装",
    (Locale::ZhHant, "install.success") => "主題已安裝",
    (Locale::JaJp, "install.success") => "テーマをインストールしました",
    (Locale::KoKr, "install.success") => "테마가 설치되었습니다",
    (Locale::DeDe, "install.success") => "Theme installiert",
    (Locale::EsEs, "install.success") => "Tema instalado",

    (Locale::EnUs, "install.error") => "Install failed",
    (Locale::ZhHans, "install.error") => "安装失败",
    (Locale::ZhHant, "install.error") => "安裝失敗",
    (Locale::JaJp, "install.error") => "インストールに失敗",
    (Locale::KoKr, "install.error") => "설치 실패",
    (Locale::DeDe, "install.error") => "Installation fehlgeschlagen",
    (Locale::EsEs, "install.error") => "Error al instalar",

    (Locale::EnUs, "install.invalid") => {
      "Not a valid theme package (JSON with format, theme, and targets.codex)"
    }
    (Locale::ZhHans, "install.invalid") => {
      "不是有效的主题包（需为含 format / theme / targets.codex 的 JSON）"
    }
    (Locale::ZhHant, "install.invalid") => {
      "不是有效的主題包（需為含 format / theme / targets.codex 的 JSON）"
    }
    (Locale::JaJp, "install.invalid") => {
      "有効なテーマパッケージではありません（format / theme / targets.codex を含む JSON）"
    }
    (Locale::KoKr, "install.invalid") => {
      "유효한 테마 패키지가 아닙니다(format, theme, targets.codex 포함 JSON)"
    }
    (Locale::DeDe, "install.invalid") => {
      "Kein gültiges Theme-Paket (JSON mit format, theme und targets.codex)"
    }
    (Locale::EsEs, "install.invalid") => {
      "No es un paquete de tema válido (JSON con format, theme y targets.codex)"
    }

    // Restore
    (Locale::EnUs, "restore.title") => "Restore Default Theme",
    (Locale::ZhHans, "restore.title") => "恢复默认主题",
    (Locale::ZhHant, "restore.title") => "還原預設主題",
    (Locale::JaJp, "restore.title") => "デフォルトテーマを復元",
    (Locale::KoKr, "restore.title") => "기본 테마 복원",
    (Locale::DeDe, "restore.title") => "Standard-Theme wiederherstellen",
    (Locale::EsEs, "restore.title") => "Restaurar tema predeterminado",

    (Locale::EnUs, "restore.subtitle") => "Undo custom theme changes and bring Codex back to stock",
    (Locale::ZhHans, "restore.subtitle") => "撤销自定义主题，将 Codex 恢复为原始外观",
    (Locale::ZhHant, "restore.subtitle") => "撤銷自訂主題，將 Codex 還原為原始外觀",
    (Locale::JaJp, "restore.subtitle") => "カスタムテーマを取り消し、Codex を標準に戻します",
    (Locale::KoKr, "restore.subtitle") => "사용자 테마를 취소하고 Codex를 기본 상태로 되돌립니다",
    (Locale::DeDe, "restore.subtitle") => {
      "Benutzerdefinierte Themes rückgängig machen und Codex zurücksetzen"
    }
    (Locale::EsEs, "restore.subtitle") => {
      "Deshacer temas personalizados y devolver Codex al estado original"
    }

    (Locale::EnUs, "restore.action") => "Restore now",
    (Locale::ZhHans, "restore.action") => "立即恢复",
    (Locale::ZhHant, "restore.action") => "立即還原",
    (Locale::JaJp, "restore.action") => "今すぐ復元",
    (Locale::KoKr, "restore.action") => "지금 복원",
    (Locale::DeDe, "restore.action") => "Jetzt wiederherstellen",
    (Locale::EsEs, "restore.action") => "Restaurar ahora",

    (Locale::EnUs, "restore.restoring") => "Restoring…",
    (Locale::ZhHans, "restore.restoring") => "恢复中…",
    (Locale::ZhHant, "restore.restoring") => "還原中…",
    (Locale::JaJp, "restore.restoring") => "復元中…",
    (Locale::KoKr, "restore.restoring") => "복원 중…",
    (Locale::DeDe, "restore.restoring") => "Wird wiederhergestellt…",
    (Locale::EsEs, "restore.restoring") => "Restaurando…",

    (Locale::EnUs, "restore.success") => "Default theme restored successfully",
    (Locale::ZhHans, "restore.success") => "已成功恢复默认主题",
    (Locale::ZhHant, "restore.success") => "已成功還原預設主題",
    (Locale::JaJp, "restore.success") => "デフォルトテーマを復元しました",
    (Locale::KoKr, "restore.success") => "기본 테마가 복원되었습니다",
    (Locale::DeDe, "restore.success") => "Standard-Theme erfolgreich wiederhergestellt",
    (Locale::EsEs, "restore.success") => "Tema predeterminado restaurado",

    (Locale::EnUs, "restore.error") => "Restore failed",
    (Locale::ZhHans, "restore.error") => "恢复失败",
    (Locale::ZhHant, "restore.error") => "還原失敗",
    (Locale::JaJp, "restore.error") => "復元に失敗しました",
    (Locale::KoKr, "restore.error") => "복원 실패",
    (Locale::DeDe, "restore.error") => "Wiederherstellung fehlgeschlagen",
    (Locale::EsEs, "restore.error") => "Error al restaurar",

    (Locale::EnUs, "restore.hint") => {
      "This will remove the currently applied custom theme package."
    }
    (Locale::ZhHans, "restore.hint") => "这将移除当前已应用的自定义主题包。",
    (Locale::ZhHant, "restore.hint") => "這將移除目前已套用的自訂主題包。",
    (Locale::JaJp, "restore.hint") => "現在適用中のカスタムテーマパッケージが削除されます。",
    (Locale::KoKr, "restore.hint") => "현재 적용된 사용자 테마 패키지가 제거됩니다.",
    (Locale::DeDe, "restore.hint") => {
      "Dadurch wird das aktuell angewendete benutzerdefinierte Theme-Paket entfernt."
    }
    (Locale::EsEs, "restore.hint") => {
      "Esto eliminará el paquete de tema personalizado aplicado actualmente."
    }

    // Settings
    (Locale::EnUs, "settings.title") => "Settings",
    (Locale::ZhHans, "settings.title") => "设置",
    (Locale::ZhHant, "settings.title") => "設定",
    (Locale::JaJp, "settings.title") => "設定",
    (Locale::KoKr, "settings.title") => "설정",
    (Locale::DeDe, "settings.title") => "Einstellungen",
    (Locale::EsEs, "settings.title") => "Ajustes",

    (Locale::EnUs, "settings.subtitle") => "Language and appearance preferences",
    (Locale::ZhHans, "settings.subtitle") => "语言与外观偏好",
    (Locale::ZhHant, "settings.subtitle") => "語言與外觀偏好",
    (Locale::JaJp, "settings.subtitle") => "言語と外観の設定",
    (Locale::KoKr, "settings.subtitle") => "언어 및 모양 환경설정",
    (Locale::DeDe, "settings.subtitle") => "Sprache und Erscheinungsbild",
    (Locale::EsEs, "settings.subtitle") => "Idioma y preferencias de apariencia",

    (Locale::EnUs, "settings.language") => "Language",
    (Locale::ZhHans, "settings.language") => "语言",
    (Locale::ZhHant, "settings.language") => "語言",
    (Locale::JaJp, "settings.language") => "言語",
    (Locale::KoKr, "settings.language") => "언어",
    (Locale::DeDe, "settings.language") => "Sprache",
    (Locale::EsEs, "settings.language") => "Idioma",

    (Locale::EnUs, "settings.language.hint") => "Choose the interface language",
    (Locale::ZhHans, "settings.language.hint") => "选择界面显示语言",
    (Locale::ZhHant, "settings.language.hint") => "選擇介面顯示語言",
    (Locale::JaJp, "settings.language.hint") => "インターフェースの表示言語を選択",
    (Locale::KoKr, "settings.language.hint") => "인터페이스 표시 언어 선택",
    (Locale::DeDe, "settings.language.hint") => "Oberflächensprache wählen",
    (Locale::EsEs, "settings.language.hint") => "Elige el idioma de la interfaz",

    (Locale::EnUs, "settings.cdp") => "Codex CDP server",
    (Locale::ZhHans, "settings.cdp") => "Codex CDP 服务",
    (Locale::ZhHant, "settings.cdp") => "Codex CDP 服務",
    (Locale::JaJp, "settings.cdp") => "Codex CDP サーバー",
    (Locale::KoKr, "settings.cdp") => "Codex CDP 서버",
    (Locale::DeDe, "settings.cdp") => "Codex-CDP-Server",
    (Locale::EsEs, "settings.cdp") => "Servidor CDP de Codex",

    (Locale::EnUs, "settings.cdp.connected") => "Connected",
    (Locale::ZhHans, "settings.cdp.connected") => "已连接",
    (Locale::ZhHant, "settings.cdp.connected") => "已連線",
    (Locale::JaJp, "settings.cdp.connected") => "接続中",
    (Locale::KoKr, "settings.cdp.connected") => "연결됨",
    (Locale::DeDe, "settings.cdp.connected") => "Verbunden",
    (Locale::EsEs, "settings.cdp.connected") => "Conectado",

    (Locale::EnUs, "settings.cdp.disconnected") => "Disconnected",
    (Locale::ZhHans, "settings.cdp.disconnected") => "未连接",
    (Locale::ZhHant, "settings.cdp.disconnected") => "未連線",
    (Locale::JaJp, "settings.cdp.disconnected") => "未接続",
    (Locale::KoKr, "settings.cdp.disconnected") => "연결 끊김",
    (Locale::DeDe, "settings.cdp.disconnected") => "Getrennt",
    (Locale::EsEs, "settings.cdp.disconnected") => "Desconectado",

    (Locale::EnUs, "settings.cdp.port") => "Port",
    (Locale::ZhHans, "settings.cdp.port") => "端口",
    (Locale::ZhHant, "settings.cdp.port") => "連接埠",
    (Locale::JaJp, "settings.cdp.port") => "ポート",
    (Locale::KoKr, "settings.cdp.port") => "포트",
    (Locale::DeDe, "settings.cdp.port") => "Port",
    (Locale::EsEs, "settings.cdp.port") => "Puerto",

    (Locale::EnUs, "settings.cdp.targets") => "Targets",
    (Locale::ZhHans, "settings.cdp.targets") => "目标页",
    (Locale::ZhHant, "settings.cdp.targets") => "目標頁",
    (Locale::JaJp, "settings.cdp.targets") => "ターゲット",
    (Locale::KoKr, "settings.cdp.targets") => "대상",
    (Locale::DeDe, "settings.cdp.targets") => "Ziele",
    (Locale::EsEs, "settings.cdp.targets") => "Destinos",

    (Locale::EnUs, "settings.cdp.hint") => "Monitors Codex remote debugging for theme injection",
    (Locale::ZhHans, "settings.cdp.hint") => "监控 Codex 远程调试端口，用于主题注入",
    (Locale::ZhHant, "settings.cdp.hint") => "監控 Codex 遠端偵錯埠，用於主題注入",
    (Locale::JaJp, "settings.cdp.hint") => "テーマ注入用の Codex リモートデバッグを監視",
    (Locale::KoKr, "settings.cdp.hint") => "테마 주입을 위한 Codex 원격 디버깅 모니터링",
    (Locale::DeDe, "settings.cdp.hint") => "Überwacht Codex-Remote-Debugging für Theme-Injection",
    (Locale::EsEs, "settings.cdp.hint") => {
      "Supervisa la depuración remota de Codex para inyección de temas"
    }

    (Locale::EnUs, "settings.cdp.port.hint") => "ChatGPT launches with this remote-debugging port",
    (Locale::ZhHans, "settings.cdp.port.hint") => "启动 ChatGPT 时使用此远程调试端口",
    (Locale::ZhHant, "settings.cdp.port.hint") => "啟動 ChatGPT 時使用此遠端偵錯埠",
    (Locale::JaJp, "settings.cdp.port.hint") => "ChatGPT はこのリモートデバッグポートで起動します",
    (Locale::KoKr, "settings.cdp.port.hint") => "ChatGPT가 이 원격 디버깅 포트로 실행됩니다",
    (Locale::DeDe, "settings.cdp.port.hint") => "ChatGPT startet mit diesem Remote-Debugging-Port",
    (Locale::EsEs, "settings.cdp.port.hint") => {
      "ChatGPT se inicia con este puerto de depuración remota"
    }

    (Locale::EnUs, "settings.cdp.port.save") => "Save & relaunch",
    (Locale::ZhHans, "settings.cdp.port.save") => "保存并重启",
    (Locale::ZhHant, "settings.cdp.port.save") => "儲存並重啟",
    (Locale::JaJp, "settings.cdp.port.save") => "保存して再起動",
    (Locale::KoKr, "settings.cdp.port.save") => "저장 후 다시 실행",
    (Locale::DeDe, "settings.cdp.port.save") => "Speichern & neu starten",
    (Locale::EsEs, "settings.cdp.port.save") => "Guardar y reiniciar",

    (Locale::EnUs, "settings.cdp.port.saved") => "Port saved",
    (Locale::ZhHans, "settings.cdp.port.saved") => "端口已保存",
    (Locale::ZhHant, "settings.cdp.port.saved") => "連接埠已儲存",
    (Locale::JaJp, "settings.cdp.port.saved") => "ポートを保存しました",
    (Locale::KoKr, "settings.cdp.port.saved") => "포트가 저장되었습니다",
    (Locale::DeDe, "settings.cdp.port.saved") => "Port gespeichert",
    (Locale::EsEs, "settings.cdp.port.saved") => "Puerto guardado",

    (Locale::EnUs, "settings.cdp.port.invalid") => "Enter a port between 1024 and 65535",
    (Locale::ZhHans, "settings.cdp.port.invalid") => "请输入 1024–65535 之间的端口",
    (Locale::ZhHant, "settings.cdp.port.invalid") => "請輸入 1024–65535 之間的連接埠",
    (Locale::JaJp, "settings.cdp.port.invalid") => "1024〜65535 のポートを入力してください",
    (Locale::KoKr, "settings.cdp.port.invalid") => "1024–65535 사이의 포트를 입력하세요",
    (Locale::DeDe, "settings.cdp.port.invalid") => "Port zwischen 1024 und 65535 eingeben",
    (Locale::EsEs, "settings.cdp.port.invalid") => "Introduce un puerto entre 1024 y 65535",

    (Locale::EnUs, "settings.theme") => "Theme",
    (Locale::ZhHans, "settings.theme") => "主题",
    (Locale::ZhHant, "settings.theme") => "主題",
    (Locale::JaJp, "settings.theme") => "テーマ",
    (Locale::KoKr, "settings.theme") => "테마",
    (Locale::DeDe, "settings.theme") => "Theme",
    (Locale::EsEs, "settings.theme") => "Tema",

    (Locale::EnUs, "settings.theme.hint") => "Switch between light and dark mode",
    (Locale::ZhHans, "settings.theme.hint") => "在浅色与深色模式之间切换",
    (Locale::ZhHant, "settings.theme.hint") => "在淺色與深色模式之間切換",
    (Locale::JaJp, "settings.theme.hint") => "ライト / ダークモードを切り替え",
    (Locale::KoKr, "settings.theme.hint") => "라이트 / 다크 모드 전환",
    (Locale::DeDe, "settings.theme.hint") => "Zwischen Hell- und Dunkelmodus wechseln",
    (Locale::EsEs, "settings.theme.hint") => "Cambiar entre modo claro y oscuro",

    (Locale::EnUs, "settings.analytics") => "Usage analytics",
    (Locale::ZhHans, "settings.analytics") => "使用分析",
    (Locale::ZhHant, "settings.analytics") => "使用分析",
    (Locale::JaJp, "settings.analytics") => "利用状況の分析",
    (Locale::KoKr, "settings.analytics") => "사용 분석",
    (Locale::DeDe, "settings.analytics") => "Nutzungsanalyse",
    (Locale::EsEs, "settings.analytics") => "Análisis de uso",

    (Locale::EnUs, "settings.analytics.hint") => "Help improve CDXTheme with anonymous product usage data",
    (Locale::ZhHans, "settings.analytics.hint") => "通过匿名产品使用数据帮助改进 CDXTheme",
    (Locale::ZhHant, "settings.analytics.hint") => "透過匿名產品使用資料協助改進 CDXTheme",
    (Locale::JaJp, "settings.analytics.hint") => "匿名の利用データで CDXTheme の改善にご協力ください",
    (Locale::KoKr, "settings.analytics.hint") => "익명 제품 사용 데이터로 CDXTheme 개선에 도움을 주세요",
    (Locale::DeDe, "settings.analytics.hint") => "Hilf, CDXTheme mit anonymen Nutzungsdaten zu verbessern",
    (Locale::EsEs, "settings.analytics.hint") => {
      "Ayuda a mejorar CDXTheme con datos anónimos de uso del producto"
    },

    (Locale::EnUs, "settings.analytics.detail") => {
      "Events like theme apply/install and app open. No account, no chat content."
    },
    (Locale::ZhHans, "settings.analytics.detail") => {
      "记录主题应用/安装、应用启动等事件。无账号、无聊天内容。"
    },
    (Locale::ZhHant, "settings.analytics.detail") => {
      "記錄主題套用/安裝、應用程式啟動等事件。無帳號、無聊天內容。"
    },
    (Locale::JaJp, "settings.analytics.detail") => {
      "テーマ適用・インストールやアプリ起動などのイベントのみ。アカウントやチャット内容は含みません。"
    },
    (Locale::KoKr, "settings.analytics.detail") => {
      "테마 적용/설치, 앱 실행 등 이벤트만 수집합니다. 계정·채팅 내용은 없습니다."
    },
    (Locale::DeDe, "settings.analytics.detail") => {
      "Ereignisse wie Theme anwenden/installieren und App-Start. Kein Konto, keine Chat-Inhalte."
    },
    (Locale::EsEs, "settings.analytics.detail") => {
      "Eventos como aplicar/instalar temas y abrir la app. Sin cuenta ni contenido de chat."
    },

    (Locale::EnUs, "settings.analytics.on") => "Analytics enabled",
    (Locale::ZhHans, "settings.analytics.on") => "已启用分析",
    (Locale::ZhHant, "settings.analytics.on") => "已啟用分析",
    (Locale::JaJp, "settings.analytics.on") => "分析を有効",
    (Locale::KoKr, "settings.analytics.on") => "분석 사용 중",
    (Locale::DeDe, "settings.analytics.on") => "Analyse aktiv",
    (Locale::EsEs, "settings.analytics.on") => "Análisis activado",

    (Locale::EnUs, "settings.analytics.off") => "Analytics disabled",
    (Locale::ZhHans, "settings.analytics.off") => "已关闭分析",
    (Locale::ZhHant, "settings.analytics.off") => "已關閉分析",
    (Locale::JaJp, "settings.analytics.off") => "分析を無効",
    (Locale::KoKr, "settings.analytics.off") => "분석 사용 안 함",
    (Locale::DeDe, "settings.analytics.off") => "Analyse deaktiviert",
    (Locale::EsEs, "settings.analytics.off") => "Análisis desactivado",

    (Locale::EnUs, "settings.analytics.saved") => "Preference saved",
    (Locale::ZhHans, "settings.analytics.saved") => "偏好已保存",
    (Locale::ZhHant, "settings.analytics.saved") => "偏好已儲存",
    (Locale::JaJp, "settings.analytics.saved") => "設定を保存しました",
    (Locale::KoKr, "settings.analytics.saved") => "환경설정이 저장되었습니다",
    (Locale::DeDe, "settings.analytics.saved") => "Einstellung gespeichert",
    (Locale::EsEs, "settings.analytics.saved") => "Preferencia guardada",

    _ => "…",
  }
}
