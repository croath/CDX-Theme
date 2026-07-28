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

    (Locale::EnUs, "nav.builder") => "Theme Builder",
    (Locale::ZhHans, "nav.builder") => "主题构建",
    (Locale::ZhHant, "nav.builder") => "主題構建",
    (Locale::JaJp, "nav.builder") => "テーマビルダー",
    (Locale::KoKr, "nav.builder") => "테마 빌더",
    (Locale::DeDe, "nav.builder") => "Theme-Builder",
    (Locale::EsEs, "nav.builder") => "Constructor",

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

    (Locale::EnUs, "settings.analytics.hint") => {
      "Help improve CDXTheme with anonymous product usage data"
    }
    (Locale::ZhHans, "settings.analytics.hint") => "通过匿名产品使用数据帮助改进 CDXTheme",
    (Locale::ZhHant, "settings.analytics.hint") => "透過匿名產品使用資料協助改進 CDXTheme",
    (Locale::JaJp, "settings.analytics.hint") => {
      "匿名の利用データで CDXTheme の改善にご協力ください"
    }
    (Locale::KoKr, "settings.analytics.hint") => {
      "익명 제품 사용 데이터로 CDXTheme 개선에 도움을 주세요"
    }
    (Locale::DeDe, "settings.analytics.hint") => {
      "Hilf, CDXTheme mit anonymen Nutzungsdaten zu verbessern"
    }
    (Locale::EsEs, "settings.analytics.hint") => {
      "Ayuda a mejorar CDXTheme con datos anónimos de uso del producto"
    }

    (Locale::EnUs, "settings.analytics.detail") => {
      "Events like theme apply/install and app open. No account, no chat content."
    }
    (Locale::ZhHans, "settings.analytics.detail") => {
      "记录主题应用/安装、应用启动等事件。无账号、无聊天内容。"
    }
    (Locale::ZhHant, "settings.analytics.detail") => {
      "記錄主題套用/安裝、應用程式啟動等事件。無帳號、無聊天內容。"
    }
    (Locale::JaJp, "settings.analytics.detail") => {
      "テーマ適用・インストールやアプリ起動などのイベントのみ。アカウントやチャット内容は含みません。"
    }
    (Locale::KoKr, "settings.analytics.detail") => {
      "테마 적용/설치, 앱 실행 등 이벤트만 수집합니다. 계정·채팅 내용은 없습니다."
    }
    (Locale::DeDe, "settings.analytics.detail") => {
      "Ereignisse wie Theme anwenden/installieren und App-Start. Kein Konto, keine Chat-Inhalte."
    }
    (Locale::EsEs, "settings.analytics.detail") => {
      "Eventos como aplicar/instalar temas y abrir la app. Sin cuenta ni contenido de chat."
    }

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

    // Theme Builder
    (Locale::EnUs, "builder.title") => "Theme Builder",
    (Locale::ZhHans, "builder.title") => "主题构建",
    (Locale::ZhHant, "builder.title") => "主題構建",
    (Locale::JaJp, "builder.title") => "テーマビルダー",
    (Locale::KoKr, "builder.title") => "테마 빌더",
    (Locale::DeDe, "builder.title") => "Theme-Builder",
    (Locale::EsEs, "builder.title") => "Constructor de temas",

    (Locale::EnUs, "builder.subtitle") => {
      "Start a build, chat with Codex CLI, and reopen saved sessions"
    }
    (Locale::ZhHans, "builder.subtitle") => "开始构建、与 Codex CLI 对话，并打开已保存会话",
    (Locale::ZhHant, "builder.subtitle") => "開始構建、與 Codex CLI 對話，並開啟已儲存工作階段",
    (Locale::JaJp, "builder.subtitle") => {
      "ビルド開始、Codex CLI とチャット、保存済みセッションを再開"
    }
    (Locale::KoKr, "builder.subtitle") => "빌드 시작, Codex CLI 채팅, 저장된 세션 다시 열기",
    (Locale::DeDe, "builder.subtitle") => {
      "Build starten, mit Codex CLI chatten und gespeicherte Sessions öffnen"
    }
    (Locale::EsEs, "builder.subtitle") => {
      "Inicia una build, chatea con Codex CLI y reabre sesiones guardadas"
    }

    (Locale::EnUs, "builder.start.title") => "New theme build",
    (Locale::ZhHans, "builder.start.title") => "新建主题构建",
    (Locale::ZhHant, "builder.start.title") => "新建主題構建",
    (Locale::JaJp, "builder.start.title") => "新しいテーマビルド",
    (Locale::KoKr, "builder.start.title") => "새 테마 빌드",
    (Locale::DeDe, "builder.start.title") => "Neuer Theme-Build",
    (Locale::EsEs, "builder.start.title") => "Nueva build de tema",

    (Locale::EnUs, "builder.start.hint") => {
      "Describe a look, generate with Codex, then apply the packed theme to Codex."
    }
    (Locale::ZhHans, "builder.start.hint") => "描述风格，用 Codex 生成主题，再应用到 Codex。",
    (Locale::ZhHant, "builder.start.hint") => "描述風格，用 Codex 產生主題，再套用到 Codex。",
    (Locale::JaJp, "builder.start.hint") => {
      "見た目を説明して Codex で生成し、完成したテーマを適用します。"
    }
    (Locale::KoKr, "builder.start.hint") => {
      "원하는 스타일을 설명하고 Codex로 생성한 뒤 테마를 적용하세요."
    }
    (Locale::DeDe, "builder.start.hint") => {
      "Beschreibe einen Look, generiere mit Codex und wende das Theme an."
    }
    (Locale::EsEs, "builder.start.hint") => {
      "Describe un estilo, genera con Codex y aplica el tema empaquetado."
    }

    (Locale::EnUs, "builder.start.action") => "New theme build",
    (Locale::ZhHans, "builder.start.action") => "新建主题构建",
    (Locale::ZhHant, "builder.start.action") => "新建主題構建",
    (Locale::JaJp, "builder.start.action") => "新しいテーマビルド",
    (Locale::KoKr, "builder.start.action") => "새 테마 빌드",
    (Locale::DeDe, "builder.start.action") => "Neuer Theme-Build",
    (Locale::EsEs, "builder.start.action") => "Nueva build de tema",

    (Locale::EnUs, "builder.generate") => "Generate",
    (Locale::ZhHans, "builder.generate") => "生成",
    (Locale::ZhHant, "builder.generate") => "產生",
    (Locale::JaJp, "builder.generate") => "生成",
    (Locale::KoKr, "builder.generate") => "생성",
    (Locale::DeDe, "builder.generate") => "Generieren",
    (Locale::EsEs, "builder.generate") => "Generar",

    (Locale::EnUs, "builder.response") => "Codex response",
    (Locale::ZhHans, "builder.response") => "Codex 回复",
    (Locale::ZhHant, "builder.response") => "Codex 回覆",
    (Locale::JaJp, "builder.response") => "Codex の応答",
    (Locale::KoKr, "builder.response") => "Codex 응답",
    (Locale::DeDe, "builder.response") => "Codex-Antwort",
    (Locale::EsEs, "builder.response") => "Respuesta de Codex",

    (Locale::EnUs, "builder.stream.empty") => {
      "Upload a hero, describe your theme, then Generate — Codex output streams here live."
    }
    (Locale::ZhHans, "builder.stream.empty") => {
      "上传主视觉并描述主题，点击生成后，Codex 输出会实时显示在这里。"
    }
    (Locale::ZhHant, "builder.stream.empty") => {
      "上傳主視覺並描述主題，點擊產生後，Codex 輸出會即時顯示在這裡。"
    }
    (Locale::JaJp, "builder.stream.empty") => {
      "ヒーローと説明を入れて生成すると、Codex の出力がここにライブ表示されます。"
    }
    (Locale::KoKr, "builder.stream.empty") => {
      "히어로와 설명을 입력하고 생성하면 Codex 출력이 여기에 실시간으로 표시됩니다."
    }
    (Locale::DeDe, "builder.stream.empty") => {
      "Hero hochladen, Theme beschreiben, Generieren — Codex-Ausgabe streamt hier live."
    }
    (Locale::EsEs, "builder.stream.empty") => {
      "Sube un hero, describe el tema y genera: la salida de Codex se muestra aquí en vivo."
    }

    (Locale::EnUs, "builder.generating") => "Building with Codex…",
    (Locale::ZhHans, "builder.generating") => "正在通过 Codex 构建…",
    (Locale::ZhHant, "builder.generating") => "正在透過 Codex 構建…",
    (Locale::JaJp, "builder.generating") => "Codex でビルド中…",
    (Locale::KoKr, "builder.generating") => "Codex로 빌드 중…",
    (Locale::DeDe, "builder.generating") => "Wird mit Codex erstellt…",
    (Locale::EsEs, "builder.generating") => "Generando con Codex…",

    (Locale::EnUs, "builder.apply") => "Apply theme",
    (Locale::ZhHans, "builder.apply") => "应用主题",
    (Locale::ZhHant, "builder.apply") => "套用主題",
    (Locale::JaJp, "builder.apply") => "テーマを適用",
    (Locale::KoKr, "builder.apply") => "테마 적용",
    (Locale::DeDe, "builder.apply") => "Theme anwenden",
    (Locale::EsEs, "builder.apply") => "Aplicar tema",

    (Locale::EnUs, "builder.applying") => "Installing and applying…",
    (Locale::ZhHans, "builder.applying") => "正在安装并应用…",
    (Locale::ZhHant, "builder.applying") => "正在安裝並套用…",
    (Locale::JaJp, "builder.applying") => "インストールして適用中…",
    (Locale::KoKr, "builder.applying") => "설치 및 적용 중…",
    (Locale::DeDe, "builder.applying") => "Installieren und anwenden…",
    (Locale::EsEs, "builder.applying") => "Instalando y aplicando…",

    (Locale::EnUs, "builder.apply.success") => "Theme installed and applied",
    (Locale::ZhHans, "builder.apply.success") => "主题已安装并应用",
    (Locale::ZhHant, "builder.apply.success") => "主題已安裝並套用",
    (Locale::JaJp, "builder.apply.success") => "テーマをインストールして適用しました",
    (Locale::KoKr, "builder.apply.success") => "테마가 설치되고 적용되었습니다",
    (Locale::DeDe, "builder.apply.success") => "Theme installiert und angewendet",
    (Locale::EsEs, "builder.apply.success") => "Tema instalado y aplicado",

    (Locale::EnUs, "builder.package.ready") => {
      "Package ready. Apply installs it into your theme library and injects it into Codex."
    }
    (Locale::ZhHans, "builder.package.ready") => {
      "主题包已就绪。点击应用将安装到主题库并注入 Codex。"
    }
    (Locale::ZhHant, "builder.package.ready") => {
      "主題包已就緒。點擊套用會安裝到主題庫並注入 Codex。"
    }
    (Locale::JaJp, "builder.package.ready") => {
      "パッケージ準備完了。適用するとテーマラブラリに入れ、Codex に注入します。"
    }
    (Locale::KoKr, "builder.package.ready") => {
      "패키지 준비됨. 적용하면 테마 라이브러리에 설치하고 Codex에 주입합니다."
    }
    (Locale::DeDe, "builder.package.ready") => {
      "Paket bereit. Anwenden installiert es in die Bibliothek und injiziert es in Codex."
    }
    (Locale::EsEs, "builder.package.ready") => {
      "Paquete listo. Aplicar lo instala en la biblioteca y lo inyecta en Codex."
    }

    (Locale::EnUs, "builder.package.missing") => {
      "Build finished, but no .cdxtheme package was found. Try generating again or ask Codex to pack to output/."
    }
    (Locale::ZhHans, "builder.package.missing") => {
      "构建已完成，但未找到 .cdxtheme 包。请重新生成，或让 Codex 打包到 output/。"
    }
    (Locale::ZhHant, "builder.package.missing") => {
      "構建已完成，但找不到 .cdxtheme 包。請重新產生，或讓 Codex 打包到 output/。"
    }
    (Locale::JaJp, "builder.package.missing") => {
      "ビルドは完了しましたが .cdxtheme が見つかりません。再生成するか、output/ へ pack してください。"
    }
    (Locale::KoKr, "builder.package.missing") => {
      "빌드는 끝났지만 .cdxtheme 패키지가 없습니다. 다시 생성하거나 output/에 pack 하세요."
    }
    (Locale::DeDe, "builder.package.missing") => {
      "Build fertig, aber kein .cdxtheme gefunden. Erneut generieren oder nach output/ packen lassen."
    }
    (Locale::EsEs, "builder.package.missing") => {
      "Build listo, pero no hay .cdxtheme. Vuelve a generar o pide a Codex empaquetar en output/."
    }

    (Locale::EnUs, "builder.generate.hint") => {
      "Upload a hero image and describe the look. Codex builds and packs a .cdxtheme from both."
    }
    (Locale::ZhHans, "builder.generate.hint") => {
      "上传主视觉图并描述风格。Codex 会据此构建并打包 .cdxtheme。"
    }
    (Locale::ZhHant, "builder.generate.hint") => {
      "上傳主視覺圖並描述風格。Codex 會據此構建並打包 .cdxtheme。"
    }
    (Locale::JaJp, "builder.generate.hint") => {
      "ヒーロー画像をアップロードし見た目を説明。Codex が .cdxtheme を生成・pack します。"
    }
    (Locale::KoKr, "builder.generate.hint") => {
      "히어로 이미지를 업로드하고 스타일을 설명하세요. Codex가 .cdxtheme을 만들고 패키징합니다."
    }
    (Locale::DeDe, "builder.generate.hint") => {
      "Hero-Bild hochladen und Look beschreiben. Codex baut und packt ein .cdxtheme."
    }
    (Locale::EsEs, "builder.generate.hint") => {
      "Sube una imagen hero y describe el estilo. Codex genera y empaqueta un .cdxtheme."
    }

    (Locale::EnUs, "builder.hero.title") => "Hero image",
    (Locale::ZhHans, "builder.hero.title") => "主视觉图",
    (Locale::ZhHant, "builder.hero.title") => "主視覺圖",
    (Locale::JaJp, "builder.hero.title") => "ヒーロー画像",
    (Locale::KoKr, "builder.hero.title") => "히어로 이미지",
    (Locale::DeDe, "builder.hero.title") => "Hero-Bild",
    (Locale::EsEs, "builder.hero.title") => "Imagen hero",

    (Locale::EnUs, "builder.hero.hint") => {
      "Required. JPEG, PNG, WebP, or GIF · max 8MB. Used as the theme home hero."
    }
    (Locale::ZhHans, "builder.hero.hint") => {
      "必填。JPEG / PNG / WebP / GIF · 最大 8MB。用作主题首页主视觉。"
    }
    (Locale::ZhHant, "builder.hero.hint") => {
      "必填。JPEG / PNG / WebP / GIF · 最大 8MB。用作主題首頁主視覺。"
    }
    (Locale::JaJp, "builder.hero.hint") => {
      "必須。JPEG / PNG / WebP / GIF · 最大 8MB。ホームのヒーローに使います。"
    }
    (Locale::KoKr, "builder.hero.hint") => {
      "필수. JPEG / PNG / WebP / GIF · 최대 8MB. 홈 히어로로 사용됩니다."
    }
    (Locale::DeDe, "builder.hero.hint") => {
      "Erforderlich. JPEG / PNG / WebP / GIF · max. 8 MB. Für den Home-Hero."
    }
    (Locale::EsEs, "builder.hero.hint") => {
      "Obligatorio. JPEG / PNG / WebP / GIF · máx. 8 MB. Se usa como hero de inicio."
    }

    (Locale::EnUs, "builder.hero.upload") => "Upload hero image",
    (Locale::ZhHans, "builder.hero.upload") => "上传主视觉图",
    (Locale::ZhHant, "builder.hero.upload") => "上傳主視覺圖",
    (Locale::JaJp, "builder.hero.upload") => "ヒーロー画像をアップロード",
    (Locale::KoKr, "builder.hero.upload") => "히어로 이미지 업로드",
    (Locale::DeDe, "builder.hero.upload") => "Hero-Bild hochladen",
    (Locale::EsEs, "builder.hero.upload") => "Subir imagen hero",

    (Locale::EnUs, "builder.hero.change") => "Change image",
    (Locale::ZhHans, "builder.hero.change") => "更换图片",
    (Locale::ZhHant, "builder.hero.change") => "更換圖片",
    (Locale::JaJp, "builder.hero.change") => "画像を変更",
    (Locale::KoKr, "builder.hero.change") => "이미지 변경",
    (Locale::DeDe, "builder.hero.change") => "Bild ändern",
    (Locale::EsEs, "builder.hero.change") => "Cambiar imagen",

    (Locale::EnUs, "builder.hero.required") => "Please upload a hero image first.",
    (Locale::ZhHans, "builder.hero.required") => "请先上传主视觉图。",
    (Locale::ZhHant, "builder.hero.required") => "請先上傳主視覺圖。",
    (Locale::JaJp, "builder.hero.required") => "先にヒーロー画像をアップロードしてください。",
    (Locale::KoKr, "builder.hero.required") => "먼저 히어로 이미지를 업로드하세요.",
    (Locale::DeDe, "builder.hero.required") => "Bitte zuerst ein Hero-Bild hochladen.",
    (Locale::EsEs, "builder.hero.required") => "Sube primero una imagen hero.",

    (Locale::EnUs, "builder.hero.invalid") => "Use a JPEG, PNG, WebP, or GIF image (max 8MB).",
    (Locale::ZhHans, "builder.hero.invalid") => "请使用 JPEG / PNG / WebP / GIF 图片（最大 8MB）。",
    (Locale::ZhHant, "builder.hero.invalid") => "請使用 JPEG / PNG / WebP / GIF 圖片（最大 8MB）。",
    (Locale::JaJp, "builder.hero.invalid") => {
      "JPEG / PNG / WebP / GIF（最大 8MB）を指定してください。"
    }
    (Locale::KoKr, "builder.hero.invalid") => {
      "JPEG / PNG / WebP / GIF 이미지(최대 8MB)를 사용하세요."
    }
    (Locale::DeDe, "builder.hero.invalid") => "JPEG / PNG / WebP / GIF (max. 8 MB) verwenden.",
    (Locale::EsEs, "builder.hero.invalid") => "Usa JPEG / PNG / WebP / GIF (máx. 8 MB).",

    (Locale::EnUs, "builder.desc.title") => "Description",
    (Locale::ZhHans, "builder.desc.title") => "描述",
    (Locale::ZhHant, "builder.desc.title") => "描述",
    (Locale::JaJp, "builder.desc.title") => "説明",
    (Locale::KoKr, "builder.desc.title") => "설명",
    (Locale::DeDe, "builder.desc.title") => "Beschreibung",
    (Locale::EsEs, "builder.desc.title") => "Descripción",

    (Locale::EnUs, "builder.desc.required") => "Please enter a short description for the theme.",
    (Locale::ZhHans, "builder.desc.required") => "请输入主题描述。",
    (Locale::ZhHant, "builder.desc.required") => "請輸入主題描述。",
    (Locale::JaJp, "builder.desc.required") => "テーマの説明を入力してください。",
    (Locale::KoKr, "builder.desc.required") => "테마 설명을 입력하세요.",
    (Locale::DeDe, "builder.desc.required") => "Bitte eine kurze Theme-Beschreibung eingeben.",
    (Locale::EsEs, "builder.desc.required") => "Introduce una descripción breve del tema.",

    (Locale::EnUs, "builder.sessions.title") => "Saved Codex sessions",
    (Locale::ZhHans, "builder.sessions.title") => "已保存的 Codex 会话",
    (Locale::ZhHant, "builder.sessions.title") => "已儲存的 Codex 工作階段",
    (Locale::JaJp, "builder.sessions.title") => "保存済み Codex セッション",
    (Locale::KoKr, "builder.sessions.title") => "저장된 Codex 세션",
    (Locale::DeDe, "builder.sessions.title") => "Gespeicherte Codex-Sessions",
    (Locale::EsEs, "builder.sessions.title") => "Sesiones de Codex guardadas",

    (Locale::EnUs, "builder.sessions.refresh") => "Refresh sessions",
    (Locale::ZhHans, "builder.sessions.refresh") => "刷新会话列表",
    (Locale::ZhHant, "builder.sessions.refresh") => "重新整理工作階段",
    (Locale::JaJp, "builder.sessions.refresh") => "セッションを更新",
    (Locale::KoKr, "builder.sessions.refresh") => "세션 새로고침",
    (Locale::DeDe, "builder.sessions.refresh") => "Sessions aktualisieren",
    (Locale::EsEs, "builder.sessions.refresh") => "Actualizar sesiones",

    (Locale::EnUs, "builder.sessions.loading") => "Loading sessions…",
    (Locale::ZhHans, "builder.sessions.loading") => "正在加载会话…",
    (Locale::ZhHant, "builder.sessions.loading") => "正在載入工作階段…",
    (Locale::JaJp, "builder.sessions.loading") => "セッションを読み込み中…",
    (Locale::KoKr, "builder.sessions.loading") => "세션 로딩 중…",
    (Locale::DeDe, "builder.sessions.loading") => "Sessions werden geladen…",
    (Locale::EsEs, "builder.sessions.loading") => "Cargando sesiones…",

    (Locale::EnUs, "builder.sessions.empty") => {
      "No Theme Builder sessions yet. Start a build — only sessions saved here and still in Codex history are listed."
    }
    (Locale::ZhHans, "builder.sessions.empty") => {
      "还没有主题构建会话。开始构建后会列出同时存在于本应用与 Codex 历史中的会话。"
    }
    (Locale::ZhHant, "builder.sessions.empty") => {
      "尚無主題構建工作階段。開始構建後會列出同時存在於本應用與 Codex 歷史中的工作階段。"
    }
    (Locale::JaJp, "builder.sessions.empty") => {
      "テーマビルドのセッションはまだありません。開始すると、本アプリと Codex 履歴の両方にあるものだけが表示されます。"
    }
    (Locale::KoKr, "builder.sessions.empty") => {
      "테마 빌더 세션이 없습니다. 빌드를 시작하면 이 앱과 Codex 기록에 모두 있는 세션만 표시됩니다."
    }
    (Locale::DeDe, "builder.sessions.empty") => {
      "Noch keine Theme-Builder-Sessions. Starte einen Build — nur Sessions in App-Daten und Codex-Verlauf erscheinen."
    }
    (Locale::EsEs, "builder.sessions.empty") => {
      "Aún no hay sesiones del constructor. Inicia una build: solo se listan las guardadas aquí y en el historial de Codex."
    }

    (Locale::EnUs, "builder.sessions.error") => "Could not load sessions.",
    (Locale::ZhHans, "builder.sessions.error") => "无法加载会话列表。",
    (Locale::ZhHant, "builder.sessions.error") => "無法載入工作階段列表。",
    (Locale::JaJp, "builder.sessions.error") => "セッションを読み込めませんでした。",
    (Locale::KoKr, "builder.sessions.error") => "세션을 불러오지 못했습니다.",
    (Locale::DeDe, "builder.sessions.error") => "Sessions konnten nicht geladen werden.",
    (Locale::EsEs, "builder.sessions.error") => "No se pudieron cargar las sesiones.",

    (Locale::EnUs, "builder.sessions.open") => "Open",
    (Locale::ZhHans, "builder.sessions.open") => "打开",
    (Locale::ZhHant, "builder.sessions.open") => "開啟",
    (Locale::JaJp, "builder.sessions.open") => "開く",
    (Locale::KoKr, "builder.sessions.open") => "열기",
    (Locale::DeDe, "builder.sessions.open") => "Öffnen",
    (Locale::EsEs, "builder.sessions.open") => "Abrir",

    (Locale::EnUs, "builder.sessions.delete") => "Delete session",
    (Locale::ZhHans, "builder.sessions.delete") => "删除会话",
    (Locale::ZhHant, "builder.sessions.delete") => "刪除工作階段",
    (Locale::JaJp, "builder.sessions.delete") => "セッションを削除",
    (Locale::KoKr, "builder.sessions.delete") => "세션 삭제",
    (Locale::DeDe, "builder.sessions.delete") => "Session löschen",
    (Locale::EsEs, "builder.sessions.delete") => "Eliminar sesión",

    (Locale::EnUs, "builder.sessions.delete.confirm") => {
      "Delete this Theme Builder session and its workspace files?"
    }
    (Locale::ZhHans, "builder.sessions.delete.confirm") => "删除此主题构建会话及其工作区文件？",
    (Locale::ZhHant, "builder.sessions.delete.confirm") => "刪除此主題構建工作階段及其工作區檔案？",
    (Locale::JaJp, "builder.sessions.delete.confirm") => {
      "この Theme Builder セッションと作業フォルダを削除しますか？"
    }
    (Locale::KoKr, "builder.sessions.delete.confirm") => {
      "이 Theme Builder 세션과 작업 공간 파일을 삭제할까요?"
    }
    (Locale::DeDe, "builder.sessions.delete.confirm") => {
      "Diese Theme-Builder-Session und den Workspace löschen?"
    }
    (Locale::EsEs, "builder.sessions.delete.confirm") => {
      "¿Eliminar esta sesión de Theme Builder y su workspace?"
    }

    (Locale::EnUs, "builder.sessions.delete.success") => "Session deleted",
    (Locale::ZhHans, "builder.sessions.delete.success") => "会话已删除",
    (Locale::ZhHant, "builder.sessions.delete.success") => "工作階段已刪除",
    (Locale::JaJp, "builder.sessions.delete.success") => "セッションを削除しました",
    (Locale::KoKr, "builder.sessions.delete.success") => "세션이 삭제되었습니다",
    (Locale::DeDe, "builder.sessions.delete.success") => "Session gelöscht",
    (Locale::EsEs, "builder.sessions.delete.success") => "Sesión eliminada",

    (Locale::EnUs, "builder.back") => "Back to sessions",
    (Locale::ZhHans, "builder.back") => "返回会话列表",
    (Locale::ZhHant, "builder.back") => "返回工作階段列表",
    (Locale::JaJp, "builder.back") => "セッション一覧へ",
    (Locale::KoKr, "builder.back") => "세션 목록으로",
    (Locale::DeDe, "builder.back") => "Zurück zu Sessions",
    (Locale::EsEs, "builder.back") => "Volver a sesiones",

    (Locale::EnUs, "builder.chat.new") => "New theme chat",
    (Locale::ZhHans, "builder.chat.new") => "新主题对话",
    (Locale::ZhHant, "builder.chat.new") => "新主題對話",
    (Locale::JaJp, "builder.chat.new") => "新しいテーマチャット",
    (Locale::KoKr, "builder.chat.new") => "새 테마 채팅",
    (Locale::DeDe, "builder.chat.new") => "Neuer Theme-Chat",
    (Locale::EsEs, "builder.chat.new") => "Nuevo chat de tema",

    (Locale::EnUs, "builder.chat.unsaved") => "New session (saved after first reply)",
    (Locale::ZhHans, "builder.chat.unsaved") => "新会话（首次回复后保存）",
    (Locale::ZhHant, "builder.chat.unsaved") => "新工作階段（首次回覆後儲存）",
    (Locale::JaJp, "builder.chat.unsaved") => "新規セッション（最初の返信後に保存）",
    (Locale::KoKr, "builder.chat.unsaved") => "새 세션 (첫 답변 후 저장)",
    (Locale::DeDe, "builder.chat.unsaved") => "Neue Session (nach erster Antwort gespeichert)",
    (Locale::EsEs, "builder.chat.unsaved") => "Nueva sesión (se guarda tras la primera respuesta)",

    (Locale::EnUs, "builder.session.loading") => "Loading session…",
    (Locale::ZhHans, "builder.session.loading") => "正在加载会话…",
    (Locale::ZhHant, "builder.session.loading") => "正在載入工作階段…",
    (Locale::JaJp, "builder.session.loading") => "セッションを読み込み中…",
    (Locale::KoKr, "builder.session.loading") => "세션 로딩 중…",
    (Locale::DeDe, "builder.session.loading") => "Session wird geladen…",
    (Locale::EsEs, "builder.session.loading") => "Cargando sesión…",

    (Locale::EnUs, "builder.session.empty") => {
      "This session has no chat messages yet. Send a message to continue."
    }
    (Locale::ZhHans, "builder.session.empty") => "此会话还没有聊天消息。发送消息以继续。",
    (Locale::ZhHant, "builder.session.empty") => "此工作階段尚無聊天訊息。傳送訊息以繼續。",
    (Locale::JaJp, "builder.session.empty") => {
      "このセッションにはまだメッセージがありません。送信して続けてください。"
    }
    (Locale::KoKr, "builder.session.empty") => {
      "이 세션에 채팅 메시지가 없습니다. 메시지를 보내 이어가세요."
    }
    (Locale::DeDe, "builder.session.empty") => {
      "Diese Session hat noch keine Nachrichten. Sende eine Nachricht zum Fortsetzen."
    }
    (Locale::EsEs, "builder.session.empty") => {
      "Esta sesión aún no tiene mensajes. Envía uno para continuar."
    }

    (Locale::EnUs, "builder.session.load_error") => "Failed to load session",
    (Locale::ZhHans, "builder.session.load_error") => "加载会话失败",
    (Locale::ZhHant, "builder.session.load_error") => "載入工作階段失敗",
    (Locale::JaJp, "builder.session.load_error") => "セッションの読み込みに失敗",
    (Locale::KoKr, "builder.session.load_error") => "세션 로드 실패",
    (Locale::DeDe, "builder.session.load_error") => "Session konnte nicht geladen werden",
    (Locale::EsEs, "builder.session.load_error") => "Error al cargar la sesión",

    (Locale::EnUs, "builder.welcome") => {
      "Describe the look you want (colors, vibe, layout). CDXTheme connects to Codex over the Agent Client Protocol (ACP) via codex-acp and streams the reply here. Needs Node/npm for the adapter (or install `codex-acp`), and `codex login` once if needed."
    }
    (Locale::ZhHans, "builder.welcome") => {
      "描述你想要的风格（颜色、氛围、布局）。CDXTheme 通过 Agent Client Protocol（ACP / codex-acp）连接 Codex，并把回复流式显示在这里。需要 Node/npm 运行适配器（或安装 codex-acp）；如需登录请执行一次 codex login。"
    }
    (Locale::ZhHant, "builder.welcome") => {
      "描述你想要的風格（顏色、氛圍、版面）。CDXTheme 透過 Agent Client Protocol（ACP / codex-acp）連接 Codex，並把回覆串流顯示在這裡。需要 Node/npm 執行適配器（或安裝 codex-acp）；如需登入請執行一次 codex login。"
    }
    (Locale::JaJp, "builder.welcome") => {
      "欲しい見た目（色・雰囲気・レイアウト）を書いてください。CDXTheme は Agent Client Protocol（ACP / codex-acp）で Codex に接続し、返信をここにストリーム表示します。アダプタには Node/npm（または codex-acp）が必要。必要なら codex login を一度。"
    }
    (Locale::KoKr, "builder.welcome") => {
      "원하는 분위기(색상, 톤, 레이아웃)를 적어 주세요. CDXTheme이 Agent Client Protocol(ACP / codex-acp)로 Codex에 연결하고 답변을 여기에 스트림합니다. 어댑터에 Node/npm(또는 codex-acp)이 필요하며, 필요하면 codex login을 한 번 하세요."
    }
    (Locale::DeDe, "builder.welcome") => {
      "Beschreibe den Look (Farben, Stimmung, Layout). CDXTheme verbindet sich über das Agent Client Protocol (ACP / codex-acp) mit Codex und streamt die Antwort hierher. Node/npm für den Adapter (oder codex-acp) nötig; bei Bedarf einmal codex login."
    }
    (Locale::EsEs, "builder.welcome") => {
      "Describe el aspecto que quieres (colores, estilo, layout). CDXTheme se conecta a Codex por Agent Client Protocol (ACP / codex-acp) y muestra la respuesta aquí. Hace falta Node/npm para el adaptador (o codex-acp); si hace falta, codex login una vez."
    }

    (Locale::EnUs, "builder.placeholder") => "Describe a theme idea…",
    (Locale::ZhHans, "builder.placeholder") => "描述一个主题想法…",
    (Locale::ZhHant, "builder.placeholder") => "描述一個主題想法…",
    (Locale::JaJp, "builder.placeholder") => "テーマのアイデアを入力…",
    (Locale::KoKr, "builder.placeholder") => "테마 아이디어를 입력…",
    (Locale::DeDe, "builder.placeholder") => "Theme-Idee beschreiben…",
    (Locale::EsEs, "builder.placeholder") => "Describe una idea de tema…",

    (Locale::EnUs, "builder.send") => "Send to Codex",
    (Locale::ZhHans, "builder.send") => "发送到 Codex",
    (Locale::ZhHant, "builder.send") => "傳送到 Codex",
    (Locale::JaJp, "builder.send") => "Codex に送信",
    (Locale::KoKr, "builder.send") => "Codex로 보내기",
    (Locale::DeDe, "builder.send") => "An Codex senden",
    (Locale::EsEs, "builder.send") => "Enviar a Codex",

    (Locale::EnUs, "builder.thinking") => "Waiting for Codex…",
    (Locale::ZhHans, "builder.thinking") => "正在等待 Codex…",
    (Locale::ZhHant, "builder.thinking") => "正在等待 Codex…",
    (Locale::JaJp, "builder.thinking") => "Codex の応答を待機中…",
    (Locale::KoKr, "builder.thinking") => "Codex 응답 대기 중…",
    (Locale::DeDe, "builder.thinking") => "Warte auf Codex…",
    (Locale::EsEs, "builder.thinking") => "Esperando a Codex…",

    (Locale::EnUs, "builder.hint") => {
      "Enter to send · Shift+Enter for newline · Codex over ACP (codex-acp)"
    }
    (Locale::ZhHans, "builder.hint") => {
      "Enter 发送 · Shift+Enter 换行 · 通过 ACP（codex-acp）连接 Codex"
    }
    (Locale::ZhHant, "builder.hint") => {
      "Enter 傳送 · Shift+Enter 換行 · 透過 ACP（codex-acp）連接 Codex"
    }
    (Locale::JaJp, "builder.hint") => {
      "Enter で送信 · Shift+Enter で改行 · ACP（codex-acp）経由で Codex"
    }
    (Locale::KoKr, "builder.hint") => {
      "Enter 전송 · Shift+Enter 줄바꿈 · ACP(codex-acp)로 Codex 연결"
    }
    (Locale::DeDe, "builder.hint") => {
      "Enter senden · Shift+Enter neue Zeile · Codex über ACP (codex-acp)"
    }
    (Locale::EsEs, "builder.hint") => {
      "Enter enviar · Shift+Enter nueva línea · Codex por ACP (codex-acp)"
    }

    (Locale::EnUs, "builder.error") => "Theme Builder",
    (Locale::ZhHans, "builder.error") => "主题构建",
    (Locale::ZhHant, "builder.error") => "主題構建",
    (Locale::JaJp, "builder.error") => "テーマビルダー",
    (Locale::KoKr, "builder.error") => "테마 빌더",
    (Locale::DeDe, "builder.error") => "Theme-Builder",
    (Locale::EsEs, "builder.error") => "Constructor de temas",

    (Locale::EnUs, "builder.install.installed") => "Theme installed",
    (Locale::ZhHans, "builder.install.installed") => "主题已安装",
    (Locale::ZhHant, "builder.install.installed") => "主題已安裝",
    (Locale::JaJp, "builder.install.installed") => "テーマをインストールしました",
    (Locale::KoKr, "builder.install.installed") => "테마가 설치되었습니다",
    (Locale::DeDe, "builder.install.installed") => "Theme installiert",
    (Locale::EsEs, "builder.install.installed") => "Tema instalado",

    (Locale::EnUs, "builder.install.applied") => "Theme installed and applied",
    (Locale::ZhHans, "builder.install.applied") => "主题已安装并应用",
    (Locale::ZhHant, "builder.install.applied") => "主題已安裝並套用",
    (Locale::JaJp, "builder.install.applied") => "テーマをインストールして適用しました",
    (Locale::KoKr, "builder.install.applied") => "테마가 설치되고 적용되었습니다",
    (Locale::DeDe, "builder.install.applied") => "Theme installiert und angewendet",
    (Locale::EsEs, "builder.install.applied") => "Tema instalado y aplicado",

    (Locale::EnUs, "builder.suggest.neon") => "Neon night",
    (Locale::ZhHans, "builder.suggest.neon") => "霓虹夜色",
    (Locale::ZhHant, "builder.suggest.neon") => "霓虹夜色",
    (Locale::JaJp, "builder.suggest.neon") => "ネオンナイト",
    (Locale::KoKr, "builder.suggest.neon") => "네온 나이트",
    (Locale::DeDe, "builder.suggest.neon") => "Neon-Nacht",
    (Locale::EsEs, "builder.suggest.neon") => "Noche neón",

    (Locale::EnUs, "builder.suggest.neon.prompt") => {
      "Design a dark Codex theme with neon cyan and magenta accents, soft glass sidebar, and a floating composer. Outline theme.json fields, CSS tokens, and key selectors for chat home and conversation."
    }
    (Locale::ZhHans, "builder.suggest.neon.prompt") => {
      "设计一个深色 Codex 主题：霓虹青与品红点缀、半透明侧边栏、悬浮输入框。给出 theme.json 字段、CSS 变量，以及聊天首页与会话页的关键选择器。"
    }
    (Locale::ZhHant, "builder.suggest.neon.prompt") => {
      "設計一個深色 Codex 主題：霓虹青與洋紅點綴、半透明側邊欄、懸浮輸入框。給出 theme.json 欄位、CSS 變數，以及聊天首頁與會話頁的關鍵選擇器。"
    }
    (Locale::JaJp, "builder.suggest.neon.prompt") => {
      "ネオンのシアンとマゼンタをアクセントにしたダーク Codex テーマを設計。ガラス風サイドバーとフローティング composer。theme.json フィールド、CSS トークン、チャットホームと会話の主要セレクタを示して。"
    }
    (Locale::KoKr, "builder.suggest.neon.prompt") => {
      "네온 시안·마젠타 액센트의 다크 Codex 테마를 설계해 줘. 글래스 사이드바와 플로팅 컴포저. theme.json 필드, CSS 토큰, 채팅 홈·대화 주요 셀렉터를 알려줘."
    }
    (Locale::DeDe, "builder.suggest.neon.prompt") => {
      "Entwirf ein dunkles Codex-Theme mit neon-cyan und magenta Akzenten, glasigem Sidebar und schwebendem Composer. Nenne theme.json-Felder, CSS-Tokens und wichtige Selektoren für Chat-Home und Konversation."
    }
    (Locale::EsEs, "builder.suggest.neon.prompt") => {
      "Diseña un tema oscuro de Codex con acentos neón cian y magenta, sidebar de cristal y composer flotante. Resume theme.json, tokens CSS y selectores clave para inicio de chat y conversación."
    }

    (Locale::EnUs, "builder.suggest.minimal") => "Soft minimal",
    (Locale::ZhHans, "builder.suggest.minimal") => "柔和极简",
    (Locale::ZhHant, "builder.suggest.minimal") => "柔和極簡",
    (Locale::JaJp, "builder.suggest.minimal") => "ソフトミニマル",
    (Locale::KoKr, "builder.suggest.minimal") => "소프트 미니멀",
    (Locale::DeDe, "builder.suggest.minimal") => "Sanft minimal",
    (Locale::EsEs, "builder.suggest.minimal") => "Minimal suave",

    (Locale::EnUs, "builder.suggest.minimal.prompt") => {
      "Create a light, soft-minimal Codex theme: warm paper background, muted sage accent, clean typography. Give a compact theme.json + codex.css starter focused on readability."
    }
    (Locale::ZhHans, "builder.suggest.minimal.prompt") => {
      "做一个浅色柔和极简 Codex 主题：暖色纸感背景、低饱和鼠尾草绿强调色、清晰排版。给出精简的 theme.json 与 codex.css 起步代码，强调可读性。"
    }
    (Locale::ZhHant, "builder.suggest.minimal.prompt") => {
      "做一個淺色柔和極簡 Codex 主題：暖色紙感背景、低飽和鼠尾草綠強調色、清晰排版。給出精簡的 theme.json 與 codex.css 起步程式碼，強調可讀性。"
    }
    (Locale::JaJp, "builder.suggest.minimal.prompt") => {
      "ライトでソフトなミニマル Codex テーマを。温かみのある紙背景、落ち着いたセージアクセント、読みやすいタイポ。簡潔な theme.json と codex.css スターターを。"
    }
    (Locale::KoKr, "builder.suggest.minimal.prompt") => {
      "라이트·소프트 미니멀 Codex 테마: 따뜻한 종이 배경, 뮤트 세이지 액센트, 가독성 좋은 타이포. 간결한 theme.json과 codex.css 스타터를 줘."
    }
    (Locale::DeDe, "builder.suggest.minimal.prompt") => {
      "Erstelle ein helles, soft-minimales Codex-Theme: warmes Papier-BG, gedämpftes Salbei-Akzent, klare Typografie. Kompaktes theme.json + codex.css Starter mit Fokus Lesbarkeit."
    }
    (Locale::EsEs, "builder.suggest.minimal.prompt") => {
      "Crea un tema Codex claro y minimalista suave: fondo papel cálido, acento salvia apagado, tipografía limpia. Da theme.json + codex.css compactos centrados en legibilidad."
    }

    (Locale::EnUs, "builder.suggest.composer") => "Fix composer",
    (Locale::ZhHans, "builder.suggest.composer") => "修输入框",
    (Locale::ZhHant, "builder.suggest.composer") => "修輸入框",
    (Locale::JaJp, "builder.suggest.composer") => "Composer 調整",
    (Locale::KoKr, "builder.suggest.composer") => "컴포저 수정",
    (Locale::DeDe, "builder.suggest.composer") => "Composer fixen",
    (Locale::EsEs, "builder.suggest.composer") => "Arreglar composer",

    (Locale::EnUs, "builder.suggest.composer.prompt") => {
      "My Codex theme breaks the chat composer on home vs conversation. Explain the correct CSS for .composer-surface-chrome (fixed on chat home, relative in thread) under html.cdxtheme-host-codex, with safe overrides."
    }
    (Locale::ZhHans, "builder.suggest.composer.prompt") => {
      "我的 Codex 主题在首页与会话页弄坏了输入框。请说明在 html.cdxtheme-host-codex 下 .composer-surface-chrome 的正确 CSS（聊天首页 fixed、会话 relative），并给出安全覆盖写法。"
    }
    (Locale::ZhHant, "builder.suggest.composer.prompt") => {
      "我的 Codex 主題在首頁與會話頁弄壞了輸入框。請說明在 html.cdxtheme-host-codex 下 .composer-surface-chrome 的正確 CSS（聊天首頁 fixed、會話 relative），並給出安全覆蓋寫法。"
    }
    (Locale::JaJp, "builder.suggest.composer.prompt") => {
      "Codex テーマでホームと会話の composer が壊れます。html.cdxtheme-host-codex 配下の .composer-surface-chrome の正しい CSS（チャットホームは fixed、スレッドは relative）と安全な上書きを説明して。"
    }
    (Locale::KoKr, "builder.suggest.composer.prompt") => {
      "Codex 테마가 홈과 대화에서 컴포저를 깨뜨려. html.cdxtheme-host-codex 아래 .composer-surface-chrome 올바른 CSS(채팅 홈 fixed, 스레드 relative)와 안전한 오버라이드를 설명해 줘."
    }
    (Locale::DeDe, "builder.suggest.composer.prompt") => {
      "Mein Codex-Theme kaputt den Composer auf Home vs. Conversation. Erkläre korrektes CSS für .composer-surface-chrome unter html.cdxtheme-host-codex (fixed auf Chat-Home, relative im Thread) mit sicheren Overrides."
    }
    (Locale::EsEs, "builder.suggest.composer.prompt") => {
      "Mi tema de Codex rompe el composer en inicio vs conversación. Explica el CSS correcto de .composer-surface-chrome bajo html.cdxtheme-host-codex (fixed en chat home, relative en hilo) con overrides seguros."
    }

    _ => "…",
  }
}
