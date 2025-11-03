import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';

export type Language = 'en' | 'es' | 'fr' | 'zh';

interface LanguageContextType {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: (key: string) => string;
}

const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

// Translation data
const translations = {
  en: {
    // Navigation
    'nav.home': 'Home',
    'nav.features': 'Features',
    'nav.demos': 'Live Demos',
    'nav.documentation': 'Documentation',
    'nav.download': 'Download',
    'nav.community': 'Community',
    'nav.education': 'Education',
    'nav.blog': 'Blog',
    'nav.about': 'About',
    'nav.contact': 'Contact',
    
    // Home page
    'hero.title': 'MultiOS - Universal Educational Operating System',
    'hero.subtitle': 'Master operating systems development through hands-on learning with cross-platform Rust-based kernel development',
    'hero.cta': 'Explore Interactive Demos',
    'hero.download': 'Download Now',
    'hero.architecture': '3 Architectures Supported',
    'hero.examples': '50,000+ Lines of Code',
    'hero.languages': '4 Languages',
    
    // Features
    'features.title': 'Comprehensive Features',
    'features.subtitle': 'Everything you need for operating systems education and development',
    'features.kernel.title': 'Modern Kernel Architecture',
    'features.kernel.desc': 'Written entirely in Rust with memory safety guarantees',
    'features.cross.title': 'Cross-Platform Support',
    'features.cross.desc': 'Native support for x86_64, ARM64, and RISC-V architectures',
    'features.edu.title': 'Educational Excellence',
    'features.edu.desc': 'Comprehensive curriculum with hands-on exercises and assessments',
    'features.community.title': 'Open Source Community',
    'features.community.desc': 'Join thousands of developers contributing to operating systems research',
    
    // Demo features
    'demo.kernel.title': 'Kernel Debugging Demo',
    'demo.process.title': 'Process Management',
    'demo.memory.title': 'Memory Allocation',
    'demo.filesystem.title': 'File System Explorer',
    
    // Footer
    'footer.description': 'MultiOS - Revolutionizing operating systems education through modern development practices',
    'footer.quicklinks': 'Quick Links',
    'footer.docs': 'Documentation',
    'footer.community': 'Community',
    'footer.education': 'Education',
    'footer.support': 'Support',
    'footer.resources': 'Resources',
    'footer.github': 'GitHub',
    'footer.discord': 'Discord',
    'footer.twitter': 'Twitter',
    'footer.newsletter': 'Newsletter',
    'footer.license': 'MIT License',
  },
  es: {
    // Navigation
    'nav.home': 'Inicio',
    'nav.features': 'Características',
    'nav.demos': 'Demostraciones',
    'nav.documentation': 'Documentación',
    'nav.download': 'Descargar',
    'nav.community': 'Comunidad',
    'nav.education': 'Educación',
    'nav.blog': 'Blog',
    'nav.about': 'Acerca de',
    'nav.contact': 'Contacto',
    
    // Home page
    'hero.title': 'MultiOS - Sistema Operativo Educativo Universal',
    'hero.subtitle': 'Domina el desarrollo de sistemas operativos mediante aprendizaje práctico con desarrollo de kernel multi-plataforma basado en Rust',
    'hero.cta': 'Explorar Demostraciones Interactivas',
    'hero.download': 'Descargar Ahora',
    'hero.architecture': '3 Arquitecturas Soportadas',
    'hero.examples': '50,000+ Líneas de Código',
    'hero.languages': '4 Idiomas',
    
    // Features
    'features.title': 'Características Completas',
    'features.subtitle': 'Todo lo que necesitas para educación y desarrollo de sistemas operativos',
    'features.kernel.title': 'Arquitectura de Kernel Moderna',
    'features.kernel.desc': 'Escrito completamente en Rust con garantías de seguridad de memoria',
    'features.cross.title': 'Soporte Multi-Plataforma',
    'features.cross.desc': 'Soporte nativo para arquitecturas x86_64, ARM64 y RISC-V',
    'features.edu.title': 'Excelencia Educativa',
    'features.edu.desc': 'Currículo completo con ejercicios prácticos y evaluaciones',
    'features.community.title': 'Comunidad de Código Abierto',
    'features.community.desc': 'Únete a miles de desarrolladores contribuyendo a la investigación de sistemas operativos',
  },
  fr: {
    // Navigation
    'nav.home': 'Accueil',
    'nav.features': 'Fonctionnalités',
    'nav.demos': 'Démos',
    'nav.documentation': 'Documentation',
    'nav.download': 'Télécharger',
    'nav.community': 'Communauté',
    'nav.education': 'Éducation',
    'nav.blog': 'Blog',
    'nav.about': 'À propos',
    'nav.contact': 'Contact',
    
    // Home page
    'hero.title': 'MultiOS - Système d\'Exploitation Éducatif Universel',
    'hero.subtitle': 'Maîtrisez le développement de systèmes d\'exploitation par l\'apprentissage pratique avec le développement de noyau multi-plateforme en Rust',
    'hero.cta': 'Explorer les Démos Interactives',
    'hero.download': 'Télécharger Maintenant',
    'hero.architecture': '3 Architectures Supportées',
    'hero.examples': '50,000+ Lignes de Code',
    'hero.languages': '4 Langues',
    
    // Features
    'features.title': 'Fonctionnalités Complètes',
    'features.subtitle': 'Tout ce dont vous avez besoin pour l\'éducation et le développement de systèmes d\'exploitation',
    'features.kernel.title': 'Architecture de Noyau Moderne',
    'features.kernel.desc': 'Écrit entièrement en Rust avec des garanties de sécurité mémoire',
    'features.cross.title': 'Support Multi-Plateforme',
    'features.cross.desc': 'Support natif pour les architectures x86_64, ARM64 et RISC-V',
    'features.edu.title': 'Excellence Éducative',
    'features.edu.desc': 'Curriculum complet avec exercices pratiques et évaluations',
    'features.community.title': 'Communauté Open Source',
    'features.community.desc': 'Rejoignez des milliers de développeurs contribuant à la recherche sur les systèmes d\'exploitation',
  },
  zh: {
    // Navigation
    'nav.home': '首页',
    'nav.features': '特性',
    'nav.demos': '演示',
    'nav.documentation': '文档',
    'nav.download': '下载',
    'nav.community': '社区',
    'nav.education': '教育',
    'nav.blog': '博客',
    'nav.about': '关于',
    'nav.contact': '联系',
    
    // Home page
    'hero.title': 'MultiOS - 通用教育操作系统',
    'hero.subtitle': '通过基于 Rust 的跨平台内核开发实践学习掌握操作系统开发',
    'hero.cta': '探索交互式演示',
    'hero.download': '立即下载',
    'hero.architecture': '支持3种架构',
    'hero.examples': '50,000+ 行代码',
    'hero.languages': '4种语言',
    
    // Features
    'features.title': '全面功能',
    'features.subtitle': '操作系统教育和开发所需的一切',
    'features.kernel.title': '现代内核架构',
    'features.kernel.desc': '完全用 Rust 编写，具有内存安全保证',
    'features.cross.title': '跨平台支持',
    'features.cross.desc': '原生支持 x86_64、ARM64 和 RISC-V 架构',
    'features.edu.title': '教育卓越',
    'features.edu.desc': '完整的课程体系，包含实践练习和评估',
    'features.community.title': '开源社区',
    'features.community.desc': '加入数千名开发者为操作系统研究做出贡献',
  },
};

interface LanguageProviderProps {
  children: ReactNode;
}

export const LanguageProvider: React.FC<LanguageProviderProps> = ({ children }) => {
  const [language, setLanguage] = useState<Language>('en');

  useEffect(() => {
    const savedLanguage = localStorage.getItem('multios-language') as Language;
    if (savedLanguage && ['en', 'es', 'fr', 'zh'].includes(savedLanguage)) {
      setLanguage(savedLanguage);
    }
  }, []);

  const handleSetLanguage = (lang: Language) => {
    setLanguage(lang);
    localStorage.setItem('multios-language', lang);
  };

  const t = (key: string): string => {
    return translations[language][key] || key;
  };

  return (
    <LanguageContext.Provider value={{ language, setLanguage: handleSetLanguage, t }}>
      {children}
    </LanguageContext.Provider>
  );
};

export const useLanguage = (): LanguageContextType => {
  const context = useContext(LanguageContext);
  if (!context) {
    throw new Error('useLanguage must be used within a LanguageProvider');
  }
  return context;
};