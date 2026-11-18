import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import fi from "@shared/i18n/fi.json";

export const defaultNS = "translation";
export const resources = {
  fi: {
    translation: fi,
  },
} as const;

i18n.use(initReactI18next).init({
  lng: "fi",
  fallbackLng: "fi",
  defaultNS,
  resources,
  interpolation: { escapeValue: false },
});

export default i18n;
