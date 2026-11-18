// Empêche l'ouverture d'une console supplémentaire sous Windows en version publiée.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    evidentyx_desktop_lib::run();
}
